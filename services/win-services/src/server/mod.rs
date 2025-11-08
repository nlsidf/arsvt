use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub interface: String,
    pub command: Vec<String>,
    pub cwd: Option<String>,
    pub credential: Option<String>,
    pub writable: bool,
    pub check_origin: bool,
    pub max_clients: usize,
    pub once: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 7681,
            interface: "0.0.0.0".to_string(),
            command: vec!["bash".to_string()],
            cwd: None,
            credential: None,
            writable: true,
            check_origin: false,
            max_clients: 0,
            once: false,
        }
    }
}

pub struct AppState {
    pub config: Config,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

pub mod websocket;
