use crate::protocol::{ClientMessage, ServerMessage};
use crate::pty::{PtyProcess, PtySize};
use crate::server::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

// 添加鼠标序列生成函数
fn generate_mouse_sequence(x: u16, y: u16, button: u8, pressed: bool) -> Vec<u8> {
    // 生成VT100/X10鼠标报告序列
    // 格式: \x1b[M<按钮状态><x坐标><y坐标>
    // 按钮状态: 0x20=左键按下, 0x21=中键按下, 0x22=右键按下, 0x23=释放
    let button_state = match (button, pressed) {
        (0, true) => 0x20,  // 左键按下
        (1, true) => 0x21,  // 中键按下
        (2, true) => 0x22,  // 右键按下
        (_, false) => 0x23, // 释放
        _ => 0x23,          // 默认释放
    };
    
    // 坐标需要加32以符合VT100规范
    let x_coord = (x + 32) as u8;
    let y_coord = (y + 32) as u8;
    
    vec![0x1b, b'M', button_state, x_coord, y_coord]
}

// 添加鼠标拖拽序列生成函数
fn generate_mouse_drag_sequence(x: u16, y: u16, button: u8) -> Vec<u8> {
    // 生成VT100/X10鼠标拖拽报告序列
    // 对于拖拽，我们使用按钮按下状态加上拖拽标志位
    let button_state = match button {
        0 => 0x60,  // 左键拖拽 (0x20 | 0x40)
        1 => 0x61,  // 中键拖拽 (0x21 | 0x40)
        2 => 0x62,  // 右键拖拽 (0x22 | 0x40)
        _ => 0x60,  // 默认左键拖拽
    };
    
    // 坐标需要加32以符合VT100规范
    let x_coord = (x + 32) as u8;
    let y_coord = (y + 32) as u8;
    
    vec![0x1b, b'M', button_state, x_coord, y_coord]
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut pty_process: Option<PtyProcess> = None;
    let mut output_rx: Option<tokio::sync::mpsc::UnboundedReceiver<Bytes>> = None;
    let mut paused = false;
    let mut initialized = false;

    info!("WebSocket connection established");

    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "localhost".to_string());

    let title_msg = ServerMessage::SetWindowTitle(format!(
        "{} ({})",
        state.config.command.join(" "),
        hostname
    ));
    if let Err(e) = sender.send(Message::Binary(title_msg.to_bytes())).await {
        error!("Failed to send window title: {}", e);
        return;
    }

    let prefs_msg = ServerMessage::SetPreferences("{}".to_string());
    if let Err(e) = sender.send(Message::Binary(prefs_msg.to_bytes())).await {
        error!("Failed to send preferences: {}", e);
        return;
    }

    loop {
        tokio::select! {
            Some(data) = async {
                if paused || !initialized {
                    None
                } else {
                    output_rx.as_mut()?.recv().await
                }
            } => {
                let msg = ServerMessage::Output(data.to_vec());
                if sender.send(Message::Binary(msg.to_bytes())).await.is_err() {
                    error!("Failed to send PTY output to client");
                    break;
                }
                debug!("Sent {} bytes of PTY output to client", data.len());
            }

            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        debug!("Received binary message from client, length: {}", data.len());
                        match ClientMessage::parse(&data) {
                            Ok(ClientMessage::Init(init)) => {
                                info!("Received Init message: cols={}, rows={}", init.columns, init.rows);
                                
                                if let Some(ref credential) = state.config.credential {
                                    if let Some(token) = init.auth_token {
                                        if &token != credential {
                                            warn!("Authentication failed");
                                            break;
                                        }
                                    } else {
                                        warn!("No auth token provided");
                                        break;
                                    }
                                }

                                let size = PtySize {
                                    cols: if init.columns > 0 { init.columns } else { 80 },
                                    rows: if init.rows > 0 { init.rows } else { 24 },
                                };

                                info!("Spawning PTY with size {}x{}", size.cols, size.rows);
                                match PtyProcess::spawn(
                                    state.config.command.clone(),
                                    size,
                                    state.config.cwd.clone(),
                                ).await {
                                    Ok((process, rx)) => {
                                        info!("PTY process spawned with PID: {}", process.pid);
                                        pty_process = Some(process);
                                        output_rx = Some(rx);
                                        initialized = true;
                                        debug!("PTY initialized, ready to receive output");
                                    }
                                    Err(e) => {
                                        error!("Failed to spawn PTY process: {}", e);
                                        break;
                                    }
                                }
                            }
                            Ok(ClientMessage::Input(data)) => {
                                debug!("Received input data from client: {:?}", data);
                                if !state.config.writable {
                                    continue;
                                }
                                
                                // Send the input to the PTY process for actual execution FIRST
                                // This is crucial for TUI applications to respond properly
                                let send_result = if let Some(ref process) = pty_process {
                                    debug!("Sending input to PTY process");
                                    let result = process.write(Bytes::from(data.clone())).await;
                                    if result.is_err() {
                                        error!("Failed to write to PTY: {}", result.as_ref().unwrap_err());
                                    } else {
                                        debug!("Input successfully sent to PTY");
                                    }
                                    result
                                } else {
                                    warn!("Received input but PTY process not initialized");
                                    Err(anyhow::anyhow!("PTY process not initialized"))
                                };
                                
                                // Echo input back to client after sending to PTY
                                // For TUI applications, we need to be more careful about echoing
                                if !data.is_empty() && send_result.is_ok() {
                                    // For TUI apps, let the application handle echoing
                                    // We only echo what the app sends back to us
                                    debug!("Input sent to PTY, waiting for app response");
                                }
                            }
                            Ok(ClientMessage::MouseClick(msg)) => {
                                debug!("Received mouse click event: x={}, y={}, button={}, pressed={}", 
                                       msg.x, msg.y, msg.button, msg.pressed);
                                if !state.config.writable {
                                    continue;
                                }
                                
                                // 将鼠标事件转换为VT100鼠标报告序列并发送到PTY
                                // 根据VT100规范生成鼠标事件序列
                                let mouse_sequence = generate_mouse_sequence(msg.x, msg.y, msg.button, msg.pressed);
                                if let Some(ref process) = pty_process {
                                    debug!("Sending mouse event to PTY process");
                                    if let Err(e) = process.write(Bytes::from(mouse_sequence)).await {
                                        error!("Failed to write mouse event to PTY: {}", e);
                                    } else {
                                        debug!("Mouse event successfully sent to PTY");
                                    }
                                } else {
                                    warn!("Received mouse event but PTY process not initialized");
                                }
                            }
                            Ok(ClientMessage::MouseDrag(msg)) => {
                                debug!("Received mouse drag event: x={}, y={}, button={}, start_x={}, start_y={}", 
                                       msg.x, msg.y, msg.button, msg.start_x, msg.start_y);
                                if !state.config.writable {
                                    continue;
                                }
                                
                                // 将鼠标拖拽事件转换为VT100鼠标报告序列并发送到PTY
                                // 对于拖拽事件，我们发送当前位置的鼠标移动事件
                                let mouse_sequence = generate_mouse_drag_sequence(msg.x, msg.y, msg.button);
                                if let Some(ref process) = pty_process {
                                    debug!("Sending mouse drag event to PTY process");
                                    if let Err(e) = process.write(Bytes::from(mouse_sequence)).await {
                                        error!("Failed to write mouse drag event to PTY: {}", e);
                                    } else {
                                        debug!("Mouse drag event successfully sent to PTY");
                                    }
                                } else {
                                    warn!("Received mouse drag event but PTY process not initialized");
                                }
                            }
                            Ok(ClientMessage::Resize { cols, rows }) => {
                                if let Some(ref process) = pty_process {
                                    let size = PtySize { cols, rows };
                                    if let Err(e) = process.resize(size).await {
                                        error!("Failed to resize PTY: {}", e);
                                    }
                                }
                            }
                            Ok(ClientMessage::Pause) => {
                                paused = true;
                                debug!("PTY output paused");
                            }
                            Ok(ClientMessage::Resume) => {
                                paused = false;
                                debug!("PTY output resumed");
                            }
                            Err(e) => {
                                warn!("Failed to parse client message: {}", e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Ok(Message::Text(_))) | Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    if let Some(process) = pty_process {
        info!("Killing PTY process {}", process.pid);
        let _ = process.kill().await;
    }

    info!("WebSocket connection closed");
}
