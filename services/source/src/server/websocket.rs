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
            }

            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
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
                                if !state.config.writable {
                                    continue;
                                }
                                if let Some(ref process) = pty_process {
                                    if let Err(e) = process.write(Bytes::from(data)).await {
                                        error!("Failed to write to PTY: {}", e);
                                    }
                                } else {
                                    warn!("Received input but PTY process not initialized");
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
