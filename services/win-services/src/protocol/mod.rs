use serde::{Deserialize, Serialize};

pub const INPUT: char = '0';
pub const RESIZE_TERMINAL: char = '1';
pub const PAUSE: char = '2';
pub const RESUME: char = '3';
pub const JSON_DATA: char = '{';
pub const MOUSE_EVENT: char = '4'; // 鼠标事件常量
pub const MOUSE_DRAG_EVENT: char = '5'; // 鼠标拖拽事件常量

pub const OUTPUT: char = '0';
pub const SET_WINDOW_TITLE: char = '1';
pub const SET_PREFERENCES: char = '2';

#[derive(Debug, Serialize, Deserialize)]
pub struct InitMessage {
    #[serde(default)]
    pub columns: u16,
    #[serde(default)]
    pub rows: u16,
    #[serde(rename = "AuthToken")]
    pub auth_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResizeMessage {
    pub columns: u16,
    pub rows: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MouseClickMessage {
    pub x: u16,
    pub y: u16,
    pub button: u8, // 0=left, 1=middle, 2=right
    pub pressed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MouseDragMessage {
    pub x: u16,
    pub y: u16,
    pub button: u8, // 0=left, 1=middle, 2=right
    pub start_x: u16,
    pub start_y: u16,
}

#[derive(Debug)]
pub enum ClientMessage {
    Input(String),
    Resize { cols: u16, rows: u16 },
    Pause,
    Resume,
    Init(InitMessage),
    MouseClick(MouseClickMessage),
    MouseDrag(MouseDragMessage), // 添加鼠标拖拽消息
}

#[derive(Debug)]
pub enum ServerMessage {
    Output(Vec<u8>),
    SetWindowTitle(String),
    SetPreferences(String),
}

impl ClientMessage {
    pub fn parse(data: &[u8]) -> anyhow::Result<Self> {
        if data.is_empty() {
            anyhow::bail!("Empty message");
        }

        let cmd = data[0] as char;
        let payload = &data[1..];

        match cmd {
            INPUT => Ok(Self::Input(String::from_utf8_lossy(payload).to_string())),
            RESIZE_TERMINAL => {
                let msg: ResizeMessage = serde_json::from_slice(payload)?;
                Ok(Self::Resize {
                    cols: msg.columns,
                    rows: msg.rows,
                })
            }
            PAUSE => Ok(Self::Pause),
            RESUME => Ok(Self::Resume),
            MOUSE_EVENT => {
                let msg: MouseClickMessage = serde_json::from_slice(payload)?;
                Ok(Self::MouseClick(msg))
            }
            MOUSE_DRAG_EVENT => {
                let msg: MouseDragMessage = serde_json::from_slice(payload)?;
                Ok(Self::MouseDrag(msg))
            }
            JSON_DATA => {
                let msg: InitMessage = serde_json::from_slice(payload)?;
                Ok(Self::Init(msg))
            }
            _ => anyhow::bail!("Unknown command: {}", cmd),
        }
    }
}

impl ServerMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Output(data) => {
                let mut msg = vec![OUTPUT as u8];
                msg.extend_from_slice(data);
                msg
            }
            Self::SetWindowTitle(title) => {
                let mut msg = vec![SET_WINDOW_TITLE as u8];
                msg.extend_from_slice(title.as_bytes());
                msg
            }
            Self::SetPreferences(prefs) => {
                let mut msg = vec![SET_PREFERENCES as u8];
                msg.extend_from_slice(prefs.as_bytes());
                msg
            }
        }
    }
}
