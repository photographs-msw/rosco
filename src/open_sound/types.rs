use serde::{Deserialize, Serialize};

/// Represents an OpenSound Control message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSoundMessage {
    pub address_pattern: String,
    pub arguments: Vec<OpenSoundArgument>,
}

/// Represents an OpenSound Control bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSoundBundle {
    pub timestamp: u64,
    pub messages: Vec<OpenSoundMessage>,
}

/// Represents different types of OSC arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpenSoundArgument {
    Int32(i32),
    Float32(f32),
    String(String),
    Blob(Vec<u8>),
    True,
    False,
    Null,
    Impulse,
}

impl OpenSoundArgument {
    /// Convert argument to f32 if possible
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            OpenSoundArgument::Float32(f) => Some(*f),
            OpenSoundArgument::Int32(i) => Some(*i as f32),
            _ => None,
        }
    }

    /// Convert argument to String if possible
    pub fn as_string(&self) -> Option<String> {
        match self {
            OpenSoundArgument::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Convert argument to i32 if possible
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            OpenSoundArgument::Int32(i) => Some(*i),
            OpenSoundArgument::Float32(f) => Some(*f as i32),
            _ => None,
        }
    }
}

/// Configuration for OpenSound server/client
#[derive(Debug, Clone)]
pub struct OpenSoundConfig {
    pub tcp_address: Option<String>,
    pub udp_address: Option<String>,
    pub sample_rate: u32,
    pub buffer_size: usize,
}

impl Default for OpenSoundConfig {
    fn default() -> Self {
        Self {
            tcp_address: Some("127.0.0.1:8000".to_string()),
            udp_address: Some("127.0.0.1:8001".to_string()),
            sample_rate: 44100,
            buffer_size: 1024,
        }
    }
} 