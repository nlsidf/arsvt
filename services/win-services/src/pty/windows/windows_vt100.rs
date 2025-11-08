use super::PtySize;
use anyhow::{Context, Result};
use bytes::Bytes;
use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc;
use vt100::Parser;
use tracing::debug;

pub struct PtyProcessInner {
    pid: u32,
    parser: Arc<Mutex<Parser>>,
}

impl PtyProcessInner {
    pub async fn spawn(
        command: Vec<String>,
        size: PtySize,
        cwd: Option<String>,
        output_tx: mpsc::UnboundedSender<Bytes>,
        mut input_rx: mpsc::UnboundedReceiver<Bytes>,
    ) -> Result<Self> {
        // 创建VT100解析器 - 改进版本以更好地支持TUI应用
        // 增加滚动缓冲区大小以支持复杂TUI应用
        let parser = Arc::new(Mutex::new(Parser::new(size.rows, size.cols, 1000))); // 增加滚动缓冲区
        let parser_clone = parser.clone();

        // 构建命令 - 改进版本以更好地支持TUI应用
        let cmd = if command.is_empty() {
            vec!["cmd.exe".to_string()]
        } else {
            command
        };

        // 启动进程 - 为TUI应用优化设置
        let mut process_builder = Command::new(&cmd[0]);
        process_builder
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            // 为TUI应用设置环境变量
            .env("TERM", "xterm-256color")  // 声明支持256色
            .env("COLORTERM", "truecolor")  // 声明支持真彩色
            .env("TERM_PROGRAM", "ttyd-rust") // 声明终端程序
            .env("TERM_PROGRAM_VERSION", "1.0") // 声明版本
            // 添加更多环境变量以支持复杂TUI应用
            .env("XTERM_VERSION", "xterm-256color") // 声明xterm兼容性
            .env("TERMINFO", "/usr/share/terminfo") // 声明terminfo路径
            .env("ANSICON", "1") // 声明ANSI控制台支持
            .env("CLICOLOR", "1") // 声明颜色输出支持
            .env("CLICOLOR_FORCE", "1"); // 强制颜色输出
            
        // 添加命令参数（如果有的话）
        if cmd.len() > 1 {
            process_builder.args(&cmd[1..]);
        }
            
        if let Some(cwd) = cwd {
            process_builder.current_dir(cwd);
        }

        let mut child = process_builder.spawn().context("Failed to spawn process")?;
        let pid = child.id();

        // 获取子进程的stdin和stdout
        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;
        let stderr = child.stderr.take().context("Failed to get stderr")?;

        // 处理输出数据 - 改进版本以更好地支持TUI应用
        let output_tx_clone = output_tx.clone();
        let parser_output = parser.clone();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut buffer = [0u8; 8192]; // 进一步增大缓冲区以提高性能
            
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let output = &buffer[..n];
                        
                        // 更新VT100解析器状态 - 对TUI应用很重要
                        let mut parser = parser_output.lock().unwrap();
                        parser.process(output);
                        
                        // 发送原始输出到客户端
                        // 对于TUI应用，我们需要确保所有VT100序列都被正确传递
                        // 特别是对于复杂的渲染序列
                        if output_tx_clone.send(Bytes::copy_from_slice(output)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from stdout: {}", e);
                        break;
                    }
                }
            }
        });

        // 处理错误输出 - 改进版本以更好地支持TUI应用
        let output_tx_clone = output_tx.clone();
        let parser_error = parser.clone();
        thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut buffer = [0u8; 8192]; // 进一步增大缓冲区以提高性能
            
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let output = &buffer[..n];
                        
                        // 更新VT100解析器状态 - 错误输出也可能包含VT100序列
                        let mut parser = parser_error.lock().unwrap();
                        parser.process(output);
                        
                        // 发送原始错误输出到客户端
                        if output_tx_clone.send(Bytes::copy_from_slice(output)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from stderr: {}", e);
                        break;
                    }
                }
            }
        });

        // 处理输入数据 - 改进版本以更好地支持TUI应用
        let mut stdin_writer = stdin;
        let output_tx_clone = output_tx.clone();
        tokio::spawn(async move {
            while let Some(data) = input_rx.recv().await {
                debug!("Windows PTY received input data: {:?}", std::str::from_utf8(&data).unwrap_or("<binary>"));
                
                // 将Bytes转换为&[u8]
                let data_slice = data.as_ref();
                
                // 对于TUI应用程序，我们需要确保所有输入都被正确处理
                // 特别是箭头键、功能键等控制序列
                let processed_data = data_slice.to_vec();
                
                // 写入到进程 - 这是关键：确保数据被正确发送到TUI程序
                if let Err(e) = stdin_writer.write_all(&processed_data) {
                    eprintln!("Failed to write to stdin: {}", e);
                    break;
                }
                
                // 立即刷新以确保TUI应用能立即收到输入
                if let Err(e) = stdin_writer.flush() {
                    eprintln!("Failed to flush stdin: {}", e);
                }
                
                // 对于TUI应用，我们需要正确回显输入
                // 但要注意不要双重处理TUI程序自己处理的控制序列
                if !processed_data.is_empty() {
                    // 检查是否是特殊控制序列
                    if processed_data.len() == 1 {
                        match processed_data[0] {
                            3 => { // Ctrl+C
                                // 不回显Ctrl+C，但仍然发送到进程
                                debug!("Ctrl+C detected, not echoing");
                            },
                            4 => { // Ctrl+D
                                // 不回显Ctrl+D，但仍然发送到进程
                                debug!("Ctrl+D detected, not echoing");
                            },
                            13 => { // 回车键
                                // 回车键需要特殊处理
                                let echo_data = vec![13, 10]; // CR + LF
                                if output_tx_clone.send(Bytes::copy_from_slice(&echo_data)).is_err() {
                                    debug!("Failed to send echo data to client");
                                }
                            },
                            _ => {
                                // 正常回显其他单字节字符
                                if output_tx_clone.send(Bytes::copy_from_slice(&processed_data)).is_err() {
                                    debug!("Failed to send echo data to client");
                                }
                            }
                        }
                    } else {
                        // 多字节序列（如箭头键、功能键等）直接回显
                        // 这些通常是VT100转义序列，TUI程序需要它们
                        if output_tx_clone.send(Bytes::copy_from_slice(&processed_data)).is_err() {
                            debug!("Failed to send echo data to client");
                        }
                    }
                }
                
                debug!("Windows PTY successfully wrote data to stdin, len: {}", processed_data.len());
            }
        });

        // 监控子进程退出
        tokio::spawn(async move {
            let _ = child.wait();
        });

        Ok(Self {
            pid,
            parser: parser_clone,
        })
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub async fn resize(&self, size: PtySize) -> Result<()> {
        let mut parser = self.parser.lock().unwrap();
        parser.set_size(size.rows, size.cols);
        Ok(())
    }

    pub async fn kill(&self) -> Result<()> {
        // 在Windows上，我们可以通过其他方式终止进程
        // 这里简单地返回Ok，实际的进程管理由操作系统处理
        Ok(())
    }
}