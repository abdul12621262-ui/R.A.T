//! Typed protocol messages for R.A.T remote desktop sessions.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEFAULT_SIGNALING_PORT: u16 = 4899;
pub const DEFAULT_RELAY_PORT: u16 = 4900;
pub const SESSION_CODE_TTL_SECS: u64 = 600;

/// Wire envelope for WebSocket signaling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum SignalingMessage {
    // Session pairing
    CreateSession,
    SessionCreated { session_code: String },
    JoinSession { session_code: String },
    LinkError { message: String },
    SessionLinked { session_code: String },
    SessionTerminated { reason: String },

    // WebRTC-style signaling relay (legacy compat)
    Signal { session_code: String, data: serde_json::Value },

    // Remote control relay
    ControlAction {
        session_code: String,
        action: ControlAction,
    },

    // Joiner consent
    ConsentRequest { admin_label: String },
    ConsentResponse { accepted: bool },

    // E2E key exchange (after pairing, before media)
    KeyExchange { public_key: Vec<u8> },
    KeyExchangeAck { public_key: Vec<u8> },

    // Encrypted media frame (JPEG/H264 blob, encrypted by rat-crypto)
    MediaFrame {
        session_code: String,
        frame: Vec<u8>,
        width: u32,
        height: u32,
        sequence: u64,
    },

    // Device identity (cloud / relay phase)
    RegisterDevice { device_id: String, name: String },
    DeviceRegistered { device_id: String },
    LookupDevice { device_id: String },
    DeviceLookupRes { found: bool, device_name: Option<String> },

    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum ControlAction {
    MouseMove { x: i32, y: i32 },
    MouseClick {
        button: MouseButton,
        double_click: bool,
    },
    TypeText { text: String },
    PressKey { key: String },
    ScreenResolution { width: u32, height: u32 },

    // Feature parity
    TerminalCmd { command: String },
    TerminalData { chunk: String },
    TerminalEnd,

    FileBrowse { path: String },
    FileBrowseRes {
        path: String,
        items: Vec<FileEntry>,
        error: Option<String>,
    },
    FileRead { path: String },
    FileReadRes {
        path: String,
        content: Option<String>,
        error: Option<String>,
    },
    FileWrite { path: String, content: String },
    FileWriteRes {
        path: String,
        success: bool,
        error: Option<String>,
    },

    ChatMessage { text: String, from: String },
    ClipboardSet { text: String },
    ClipboardGet,
    ClipboardData { text: String },

    SystemStatsRequest,
    SystemStatsRes { stats: SystemStats },

    Disconnect,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub os_type: String,
    pub hostname: String,
    pub cpu_usage: u32,
    pub memory_usage: u32,
    pub uptime_secs: u64,
    pub total_memory: u64,
    pub free_memory: u64,
}

#[derive(Debug, Clone)]
pub struct SessionRoom {
    pub code: String,
    pub admin_id: Uuid,
    pub joiner_id: Option<Uuid>,
    pub created_at: std::time::Instant,
}

impl SessionRoom {
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() > SESSION_CODE_TTL_SECS
    }
}

/// Generate a user-friendly session code (###-###).
pub fn generate_session_code() -> String {
    let raw = (100_000 + (rand_u32() % 900_000)).to_string();
    format!("{}-{}", &raw[..3], &raw[3..])
}

fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    nanos ^ (nanos >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_code_format() {
        let code = generate_session_code();
        assert_eq!(code.len(), 7);
        assert_eq!(code.as_bytes()[3], b'-');
    }

    #[test]
    fn message_roundtrip() {
        let msg = SignalingMessage::SessionCreated {
            session_code: "123-456".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: SignalingMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, SignalingMessage::SessionCreated { .. }));
    }
}
