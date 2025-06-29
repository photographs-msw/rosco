use crate::open_sound::server::OpenSoundServer;
use crate::open_sound::handler::{MusicalNoteHandler, RouteHandler};
use crate::open_sound::types::OpenSoundConfig;

/// Example OpenSound server for musical notes
pub async fn run_server() {
    println!("Starting OpenSound server example...");
    
    // Create server configuration
    let config = OpenSoundConfig {
        tcp_address: Some("127.0.0.1:8000".to_string()),
        udp_address: Some("127.0.0.1:8001".to_string()),
        sample_rate: 44100,
        buffer_size: 1024,
    };
    
    // Create server
    let mut server = OpenSoundServer::new(config);
    
    // Add route handlers
    let note_handler = MusicalNoteHandler::new();
    server.add_handler("/note/oscillator", Box::new(note_handler.clone()));
    server.add_handler("/note/sample", Box::new(note_handler.clone()));
    server.add_handler("/note/play", Box::new(note_handler.clone()));
    
    // Add wildcard handler for all note-related messages
    server.add_handler("/note/*", Box::new(note_handler));
    
    println!("OpenSound server running on:");
    println!("  TCP: 127.0.0.1:8000");
    println!("  UDP: 127.0.0.1:8001");
    println!("");
    println!("Available routes:");
    println!("  /note/oscillator - Create oscillator note (frequency, volume, start_time, duration, waveforms)");
    println!("  /note/sample     - Create sample note (file_path, volume, start_time, duration)");
    println!("  /note/play       - Play note immediately (note_type, ...)");
    println!("");
    println!("Press Ctrl+C to stop the server");
    
    // Run the server
    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
    }
}

/// Example of how to use the server programmatically
pub async fn run_server_with_custom_config() {
    println!("Starting OpenSound server with custom configuration...");
    
    // Custom configuration
    let config = OpenSoundConfig {
        tcp_address: Some("127.0.0.1:9000".to_string()),
        udp_address: Some("127.0.0.1:9001".to_string()),
        sample_rate: 48000,
        buffer_size: 2048,
    };
    
    let mut server = OpenSoundServer::new(config);
    
    // Add handlers
    let note_handler = MusicalNoteHandler::new();
    server.add_handler("/note/*", Box::new(note_handler));
    
    println!("Custom OpenSound server running on:");
    println!("  TCP: 127.0.0.1:9000");
    println!("  UDP: 127.0.0.1:9001");
    
    // Run for a limited time
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    server.stop();
    println!("Server stopped after 30 seconds");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let config = OpenSoundConfig::default();
        let mut server = OpenSoundServer::new(config);
        
        let note_handler = MusicalNoteHandler::new();
        server.add_handler("/note/oscillator", Box::new(note_handler));
        
        // Test that server can be created and configured
        assert!(!server.is_running());
    }
} 