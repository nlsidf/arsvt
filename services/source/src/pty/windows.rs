use super::PtySize;
use anyhow::{Context, Result};
use bytes::Bytes;
use std::ffi::CString;
use std::os::windows::io::FromRawHandle;
use std::ptr;
use tokio::sync::mpsc;
use windows::Win32::Foundation::*;
use windows::Win32::Security::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Console::*;
use windows::Win32::System::Pipes::*;
use windows::Win32::System::Threading::*;

static mut PIPE_COUNTER: u32 = 0;

pub struct PtyProcessInner {
    pid: u32,
    process_handle: HANDLE,
    pseudo_console: HPCON,
}

impl PtyProcessInner {
    pub async fn spawn(
        command: Vec<String>,
        size: PtySize,
        cwd: Option<String>,
        output_tx: mpsc::UnboundedSender<Bytes>,
        mut input_rx: mpsc::UnboundedReceiver<Bytes>,
    ) -> Result<Self> {
        unsafe {
            // 获取唯一的管道计数器
            let counter = PIPE_COUNTER;
            PIPE_COUNTER += 1;
            let pid = std::process::id();

            // 创建命名管道名称
            let in_pipe_name = format!("\\\\.\\pipe\\ttyd-rust-in-{}-{}", pid, counter);
            let out_pipe_name = format!("\\\\.\\pipe\\ttyd-rust-out-{}-{}", pid, counter);

            let in_pipe_name_wide: Vec<u16> = in_pipe_name.encode_utf16().chain(std::iter::once(0)).collect();
            let out_pipe_name_wide: Vec<u16> = out_pipe_name.encode_utf16().chain(std::iter::once(0)).collect();

            // 创建命名管道
            let sa = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: ptr::null_mut(),
                bInheritHandle: FALSE,
            };

            let in_pipe = CreateNamedPipeW(
                windows::core::PCWSTR(in_pipe_name_wide.as_ptr()),
                FILE_FLAG_FIRST_PIPE_INSTANCE | PIPE_ACCESS_INBOUND | PIPE_ACCESS_OUTBOUND,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                1,
                0,
                0,
                30000,
                Some(&sa),
            );

            let out_pipe = CreateNamedPipeW(
                windows::core::PCWSTR(out_pipe_name_wide.as_ptr()),
                FILE_FLAG_FIRST_PIPE_INSTANCE | PIPE_ACCESS_INBOUND | PIPE_ACCESS_OUTBOUND,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                1,
                0,
                0,
                30000,
                Some(&sa),
            );

            if in_pipe.is_invalid() || out_pipe.is_invalid() {
                return Err(anyhow::anyhow!("Failed to create named pipes"));
            }

            let coord = COORD {
                X: size.cols as i16,
                Y: size.rows as i16,
            };

            // 创建 ConPTY
            let pseudo_console = CreatePseudoConsole(coord, in_pipe, out_pipe, 0)
                .context("Failed to create pseudo console")?;

            // 关闭原始管道句柄（ConPTY 已经持有）
            CloseHandle(in_pipe).ok();
            CloseHandle(out_pipe).ok();

            // 设置进程属性
            let mut startup_info_ex: STARTUPINFOEXW = std::mem::zeroed();
            startup_info_ex.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;

            let mut attribute_list_size: usize = 0;
            InitializeProcThreadAttributeList(
                LPPROC_THREAD_ATTRIBUTE_LIST(ptr::null_mut()),
                1,
                0,
                &mut attribute_list_size,
            )
            .ok();

            let mut attribute_list = vec![0u8; attribute_list_size];
            let attribute_list_ptr = LPPROC_THREAD_ATTRIBUTE_LIST(attribute_list.as_mut_ptr() as *mut _);

            InitializeProcThreadAttributeList(attribute_list_ptr, 1, 0, &mut attribute_list_size)
                .context("Failed to initialize proc thread attribute list")?;

            UpdateProcThreadAttribute(
                attribute_list_ptr,
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                Some(&pseudo_console as *const _ as *const _),
                std::mem::size_of::<HPCON>(),
                None,
                None,
            )
            .context("Failed to update proc thread attribute")?;

            startup_info_ex.lpAttributeList = attribute_list_ptr;

            let cmd_line = if command.is_empty() {
                "cmd.exe".to_string()
            } else if command.len() == 1 {
                command[0].clone()
            } else {
                format!("{} {}", command[0], command[1..].join(" "))
            };

            let mut cmd_line_wide: Vec<u16> = cmd_line.encode_utf16().chain(std::iter::once(0)).collect();

            let cwd_wide = cwd.as_ref().map(|s| {
                s.encode_utf16()
                    .chain(std::iter::once(0))
                    .collect::<Vec<u16>>()
            });

            let cwd_ptr = cwd_wide
                .as_ref()
                .map(|v| v.as_ptr())
                .unwrap_or(ptr::null());

            let mut process_info: PROCESS_INFORMATION = std::mem::zeroed();

            CreateProcessW(
                None,
                windows::core::PWSTR(cmd_line_wide.as_mut_ptr()),
                None,
                None,
                false,
                EXTENDED_STARTUPINFO_PRESENT,
                None,
                windows::core::PCWSTR(cwd_ptr),
                &startup_info_ex.StartupInfo,
                &mut process_info,
            )
            .context("Failed to create process")?;

            DeleteProcThreadAttributeList(attribute_list_ptr);

            CloseHandle(process_info.hThread).ok();

            let pid = process_info.dwProcessId;
            let process_handle = process_info.hProcess;

            // 连接到命名管道
            tokio::task::spawn_blocking({
                let in_pipe_name = in_pipe_name.clone();
                let out_pipe_name = out_pipe_name.clone();
                let output_tx = output_tx.clone();

                move || {
                    use std::io::{Read, Write};
                    use windows::Win32::Storage::FileSystem::{CreateFileA, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL};

                    unsafe {
                        let in_name_cstr = CString::new(in_pipe_name.as_str()).unwrap();
                        let out_name_cstr = CString::new(out_pipe_name.as_str()).unwrap();

                        let in_handle = CreateFileA(
                            windows::core::PCSTR(in_name_cstr.as_ptr() as *const u8),
                            GENERIC_WRITE.0,
                            FILE_SHARE_NONE,
                            None,
                            OPEN_EXISTING,
                            FILE_ATTRIBUTE_NORMAL,
                            None,
                        ).unwrap_or(INVALID_HANDLE_VALUE);

                        let out_handle = CreateFileA(
                            windows::core::PCSTR(out_name_cstr.as_ptr() as *const u8),
                            GENERIC_READ.0,
                            FILE_SHARE_NONE,
                            None,
                            OPEN_EXISTING,
                            FILE_ATTRIBUTE_NORMAL,
                            None,
                        ).unwrap_or(INVALID_HANDLE_VALUE);

                        if in_handle == INVALID_HANDLE_VALUE || out_handle == INVALID_HANDLE_VALUE {
                            eprintln!("Failed to connect to named pipes");
                            return;
                        }

                        let mut in_file = std::fs::File::from_raw_handle(in_handle.0 as _);
                        let mut out_file = std::fs::File::from_raw_handle(out_handle.0 as _);

                        std::thread::spawn(move || {
                            let mut buffer = vec![0u8; 8192];
                            loop {
                                match out_file.read(&mut buffer) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        if output_tx.send(Bytes::copy_from_slice(&buffer[..n])).is_err() {
                                            break;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        });

                        // 输入处理在当前线程
                        let runtime = tokio::runtime::Handle::current();
                        loop {
                            let data = runtime.block_on(async { input_rx.recv().await });
                            match data {
                                Some(data) => {
                                    if in_file.write_all(&data).is_err() {
                                        break;
                                    }
                                }
                                None => break,
                            }
                        }
                    }
                }
            });

            tokio::spawn(async move {
                WaitForSingleObject(process_handle, INFINITE);
                CloseHandle(process_handle).ok();
            });

            Ok(Self {
                pid,
                process_handle,
                pseudo_console,
            })
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub async fn resize(&self, size: PtySize) -> Result<()> {
        unsafe {
            let coord = COORD {
                X: size.cols as i16,
                Y: size.rows as i16,
            };
            ResizePseudoConsole(self.pseudo_console, coord)
                .context("Failed to resize pseudo console")?;
        }
        Ok(())
    }

    pub async fn kill(&self) -> Result<()> {
        unsafe {
            TerminateProcess(self.process_handle, 1)
                .context("Failed to terminate process")?;
        }
        Ok(())
    }
}

impl Drop for PtyProcessInner {
    fn drop(&mut self) {
        unsafe {
            ClosePseudoConsole(self.pseudo_console);
        }
    }
}
