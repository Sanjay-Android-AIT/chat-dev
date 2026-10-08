use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub timestamp: String,
    pub content: String,
    pub is_direct: bool,
}

impl ChatMessage {
    pub fn new(sender: impl Into<String>, content: impl Into<String>) -> Self {
        let now = chrono::Local::now();
        let rand_suffix: u32 = rand::random();
        Self {
            id: format!("{:x}-{:x}", now.timestamp_millis(), rand_suffix),
            sender: sender.into(),
            timestamp: now.format("%H:%M:%S").to_string(),
            content: content.into(),
            is_direct: false,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| e.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkPacket {
    Chat(ChatMessage),
    Presence {
        node_id: String,
        nick: String,
    },
}

impl NetworkPacket {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        // Try NetworkPacket first
        if let Ok(packet) = serde_json::from_slice::<NetworkPacket>(bytes) {
            return Ok(packet);
        }
        // Fallback for ChatMessage direct payload
        if let Ok(msg) = serde_json::from_slice::<ChatMessage>(bytes) {
            return Ok(NetworkPacket::Chat(msg));
        }
        Err("Failed to parse packet payload".into())
    }
}
