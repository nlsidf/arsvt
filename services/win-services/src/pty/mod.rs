use anyhow::{Context, Result};
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;

#[derive(Debug, Clone)]
pub struct PtySize {
    pub cols: u16,
    pub rows: u16,
}

impl Default for PtySize {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
        }
    }
}

pub struct PtyProcess {
    pub pid: u32,
    tx: mpsc::UnboundedSender<Bytes>,
    inner: Arc<PtyProcessInner>,
}

impl PtyProcess {
    pub async fn spawn(
        command: Vec<String>,
        size: PtySize,
        cwd: Option<String>,
    ) -> Result<(Self, mpsc::UnboundedReceiver<Bytes>)> {
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        let (input_tx, input_rx) = mpsc::unbounded_channel();

        let inner = PtyProcessInner::spawn(command, size, cwd, output_tx, input_rx)
            .await
            .context("Failed to spawn PTY process")?;

        let process = Self {
            pid: inner.pid(),
            tx: input_tx,
            inner: Arc::new(inner),
        };

        Ok((process, output_rx))
    }

    pub async fn write(&self, data: Bytes) -> Result<()> {
        debug!("PTY writing data: {:?}", std::str::from_utf8(data.as_ref()).unwrap_or("<binary>"));
        self.tx
            .send(data)
            .map_err(|_| anyhow::anyhow!("Failed to send data to PTY"))?;
        Ok(())
    }

    pub async fn resize(&self, size: PtySize) -> Result<()> {
        self.inner.resize(size).await
    }

    pub async fn kill(&self) -> Result<()> {
        self.inner.kill().await
    }
}
