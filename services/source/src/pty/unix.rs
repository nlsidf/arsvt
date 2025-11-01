use super::PtySize;
use anyhow::{Context, Result};
use bytes::Bytes;
use nix::pty::{forkpty, Winsize};
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::waitpid;
use nix::unistd::{chdir, execvp, ForkResult, Pid};
use std::env;
use std::ffi::CString;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::io::IntoRawFd;
use tokio::sync::mpsc;
use tokio::task;
use std::os::unix::io::FromRawFd;

pub struct PtyProcessInner {
    pid: Pid,
    master_fd: RawFd,
}

impl PtyProcessInner {
    pub async fn spawn(
        command: Vec<String>,
        size: PtySize,
        cwd: Option<String>,
        output_tx: mpsc::UnboundedSender<Bytes>,
        mut input_rx: mpsc::UnboundedReceiver<Bytes>,
    ) -> Result<Self> {
        let winsize = Winsize {
            ws_row: size.rows,
            ws_col: size.cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let result = unsafe { forkpty(Some(&winsize), None)? };

        match result.fork_result {
            ForkResult::Parent { child } => {
                let master = result.master;
                let master_fd = master.as_raw_fd();

                let master_fd_raw = master.into_raw_fd();
                
                tokio::spawn(async move {
                    use tokio::io::unix::AsyncFd;
                    use std::io::{Read, Write};
                    
                    let mut master_file = unsafe { std::fs::File::from_raw_fd(master_fd_raw) };
                    let async_fd = AsyncFd::new(master_fd_raw).unwrap();
                    
                    let mut buffer = vec![0u8; 8192];
                    loop {
                        tokio::select! {
                            Ok(mut guard) = async_fd.readable() => {
                                match master_file.read(&mut buffer) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        guard.clear_ready();
                                        if output_tx.send(Bytes::copy_from_slice(&buffer[..n])).is_err() {
                                            break;
                                        }
                                    }
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                        guard.clear_ready();
                                    }
                                    Err(e) => {
                                        eprintln!("PTY read error: {}", e);
                                        break;
                                    }
                                }
                            }
                            Some(data) = input_rx.recv() => {
                                if let Err(e) = master_file.write_all(&data) {
                                    eprintln!("PTY write error: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                });

                task::spawn(async move {
                    let _ = waitpid(child, None);
                });

                Ok(Self {
                    pid: child,
                    master_fd,
                })
            }
            ForkResult::Child => {
                if let Some(dir) = cwd {
                    let _ = chdir(dir.as_str());
                }

                env::set_var("TERM", "xterm-256color");

                let args: Vec<CString> = command
                    .iter()
                    .map(|s| CString::new(s.as_str()).unwrap())
                    .collect();

                let _ = execvp(&args[0], &args);
                std::process::exit(1);
            }
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid.as_raw() as u32
    }

    pub async fn resize(&self, size: PtySize) -> Result<()> {
        let winsize = Winsize {
            ws_row: size.rows,
            ws_col: size.cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        nix::ioctl_write_ptr_bad!(tiocswinsz, libc::TIOCSWINSZ, Winsize);
        unsafe {
            tiocswinsz(self.master_fd, &winsize as *const Winsize)
                .context("Failed to resize PTY")?;
        }

        Ok(())
    }

    pub async fn kill(&self) -> Result<()> {
        kill(self.pid, Signal::SIGTERM).context("Failed to kill process")?;
        Ok(())
    }
}
