//! OpenSound Control (OSC) Protocol Implementation
//! 
//! This module provides a complete implementation of the OpenSound Control protocol
//! for musical note generation and audio synthesis. It integrates with the existing
//! audio system to convert OSC messages into PlaybackNote objects that can be
//! processed by the audio generation system.
//! 
//! ## Features
//! 
//! - **TCP and UDP Support**: Both transport protocols are supported
//! - **Message Parsing**: Full OSC message and bundle parsing
//! - **Musical Integration**: Direct integration with the audio synthesis system
//! - **Async Support**: Built on tokio for high-performance async I/O
//! - **Type Safety**: Strongly typed message arguments and error handling
//! 
//! ## Quick Start
//! 
//! ```rust
//! use crate::open_sound::{OpenSoundServer, MusicalNoteHandler};
//! 
//! #[tokio::main]
//! async fn main() {
//!     let mut server = OpenSoundServer::default();
//!     server.add_handler("/note/*", Box::new(MusicalNoteHandler::new()));
//!     server.run().await.unwrap();
//! }
//! ```
//! 
//! ## Message Format
//! 
//! ### Oscillator Note
//! ```
//! /note/oscillator frequency volume start_time_ms duration_ms waveforms
//! ```
//! 
//! ### Sample Note
//! ```
//! /note/sample file_path volume start_time_ms duration_ms
//! ```
//! 
//! ### Immediate Play
//! ```
//! /note/play note_type frequency volume start_time_ms duration_ms waveforms
//! ```

pub mod types;
pub mod error;
pub mod message;
pub mod handler;
pub mod server;
pub mod client;
pub mod integration;
pub mod open_sound_utils;
pub mod examples;

// Re-export main types for convenience
pub use types::{OpenSoundConfig};
pub use handler::{MusicalNoteHandler};
pub use server::OpenSoundServer;
pub use client::{OpenSoundClient};
pub use integration::OpenSoundIntegration;

// Re-export examples for easy access
pub use examples::server_example;
pub use examples::client_example;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default TCP port
pub const DEFAULT_TCP_PORT: u16 = 8000;

/// Default UDP port  
pub const DEFAULT_UDP_PORT: u16 = 8001;

/// Default sample rate
pub const DEFAULT_SAMPLE_RATE: u32 = 44100;

/// Default buffer size
pub const DEFAULT_BUFFER_SIZE: usize = 1024;

/// Create a default server configuration
pub fn default_config() -> OpenSoundConfig {
    OpenSoundConfig {
        tcp_address: Some(format!("127.0.0.1:{}", DEFAULT_TCP_PORT)),
        udp_address: Some(format!("127.0.0.1:{}", DEFAULT_UDP_PORT)),
        sample_rate: DEFAULT_SAMPLE_RATE,
        buffer_size: DEFAULT_BUFFER_SIZE,
    }
}

/// Create a server with default configuration and musical note handlers
pub fn create_musical_server() -> OpenSoundServer {
    let mut server = OpenSoundServer::new(default_config());
    let note_handler = MusicalNoteHandler::new();
    
    server.add_handler("/note/oscillator", Box::new(note_handler.clone()));
    server.add_handler("/note/sample", Box::new(note_handler.clone()));
    server.add_handler("/note/play", Box::new(note_handler.clone()));
    server.add_handler("/note/*", Box::new(note_handler));
    
    server
}

/// Create a client with default configuration
pub fn create_client() -> OpenSoundClient {
    OpenSoundClient::new(default_config())
}

/// Create an integration instance
pub fn create_integration() -> OpenSoundIntegration {
    OpenSoundIntegration::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert_eq!(config.sample_rate, DEFAULT_SAMPLE_RATE);
        assert_eq!(config.buffer_size, DEFAULT_BUFFER_SIZE);
    }

    #[test]
    fn test_create_musical_server() {
        let server = create_musical_server();
        assert!(!server.is_running());
    }

    #[test]
    fn test_create_client() {
        let client = create_client();
        assert!(!client.is_tcp_connected());
        assert!(!client.is_udp_connected());
    }

    #[test]
    fn test_create_integration() {
        let integration = create_integration();
        assert_eq!(integration.queued_note_count(), 0);
    }
} 