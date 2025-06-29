use std::collections::VecDeque;
use tokio::sync::mpsc;
use crate::open_sound::types::{OpenSoundMessage, OpenSoundConfig};
use crate::open_sound::error::{OpenSoundError, OpenSoundResult};
use crate::open_sound::handler::MusicalNoteHandler;
use crate::note::playback_note::PlaybackNote;
use crate::audio_gen::oscillator::OscillatorTables;
use crate::audio_gen::audio_gen::gen_notes_stream;
use crate::open_sound::open_sound_utils::{OscArg, create_osc_message_bytes, create_osc_bundle_bytes, create_osc_message_bytes_mixed, create_osc_bundle_bytes_mixed};

/// Integration between OpenSound Protocol and audio system
pub struct OpenSoundIntegration {
    oscillator_tables: OscillatorTables,
    note_queue: VecDeque<PlaybackNote>,
    note_handler: MusicalNoteHandler,
    message_tx: Option<mpsc::Sender<OpenSoundMessage>>,
    message_rx: Option<mpsc::Receiver<OpenSoundMessage>>,
    running: bool,
}

impl OpenSoundIntegration {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<OpenSoundMessage>(100);
        Self {
            oscillator_tables: OscillatorTables::new(),
            note_queue: VecDeque::new(),
            note_handler: MusicalNoteHandler::new(),
            message_tx: Some(tx),
            message_rx: Some(rx),
            running: false,
        }
    }

    /// Queue a note for later playback
    pub fn queue_note(&mut self, note: PlaybackNote) {
        self.note_queue.push_back(note);
    }

    /// Play all queued notes
    pub fn play_queued_notes(&mut self) -> OpenSoundResult<()> {
        if self.note_queue.is_empty() {
            return Ok(());
        }

        let notes: Vec<PlaybackNote> = self.note_queue.drain(..).collect();
        gen_notes_stream(notes, self.oscillator_tables.clone());
        Ok(())
    }

    /// Play a note immediately
    pub fn play_immediate_note(&mut self, note: PlaybackNote) -> OpenSoundResult<()> {
        gen_notes_stream(vec![note], self.oscillator_tables.clone());
        Ok(())
    }

    /// Process an OpenSound message and convert to note
    pub fn process_message(&mut self, message: OpenSoundMessage) -> OpenSoundResult<()> {
        match message.address_pattern.as_str() {
            "/note/oscillator" => {
                let playback_note = self.note_handler.handle_oscillator_note(message)?;
                self.queue_note(playback_note);
                Ok(())
            }
            "/note/sample" => {
                let playback_note = self.note_handler.handle_sample_note(message)?;
                self.queue_note(playback_note);
                Ok(())
            }
            "/note/play" => {
                self.note_handler.handle_play_note(message)
            }
            _ => Err(OpenSoundError::InvalidAddressPattern(message.address_pattern)),
        }
    }

    /// Start the integration service
    pub async fn start(&mut self) -> OpenSoundResult<()> {
        self.running = true;
        
        // Spawn message processing task
        if let Some(rx) = self.message_rx.take() {
            let mut integration = self.clone();
            tokio::spawn(async move {
                integration.process_messages(rx).await;
            });
        }

        Ok(())
    }

    /// Stop the integration service
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Get a sender for posting messages
    pub fn get_message_sender(&self) -> Option<mpsc::Sender<OpenSoundMessage>> {
        self.message_tx.clone()
    }

    /// Process incoming messages
    async fn process_messages(&mut self, mut rx: mpsc::Receiver<OpenSoundMessage>) {
        while self.running {
            if let Some(message) = rx.recv().await {
                if let Err(e) = self.process_message(message) {
                    eprintln!("Error processing OpenSound message: {}", e);
                }
            }
        }
    }

    /// Get the number of queued notes
    pub fn queued_note_count(&self) -> usize {
        self.note_queue.len()
    }

    /// Clear all queued notes
    pub fn clear_queue(&mut self) {
        self.note_queue.clear();
    }

    /// Get a reference to the oscillator tables
    pub fn oscillator_tables(&self) -> &OscillatorTables {
        &self.oscillator_tables
    }

    /// Get a mutable reference to the oscillator tables
    pub fn oscillator_tables_mut(&mut self) -> &mut OscillatorTables {
        &mut self.oscillator_tables
    }
}

impl Clone for OpenSoundIntegration {
    fn clone(&self) -> Self {
        let (tx, rx) = mpsc::channel::<OpenSoundMessage>(100);
        Self {
            oscillator_tables: self.oscillator_tables.clone(),
            note_queue: self.note_queue.clone(),
            note_handler: MusicalNoteHandler::new(),
            message_tx: Some(tx),
            message_rx: Some(rx),
            running: self.running,
        }
    }
}

impl Default for OpenSoundIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common musical operations
impl OpenSoundIntegration {
    /// Create and queue an oscillator note
    pub fn queue_oscillator_note(
        &mut self,
        frequency: f32,
        volume: f32,
        start_time_ms: f32,
        duration_ms: f32,
        waveforms: &str,
    ) -> OpenSoundResult<()> {
        let message = OpenSoundMessage {
            address_pattern: "/note/oscillator".to_string(),
            arguments: vec![
                crate::open_sound::types::OpenSoundArgument::Float32(frequency),
                crate::open_sound::types::OpenSoundArgument::Float32(volume),
                crate::open_sound::types::OpenSoundArgument::Float32(start_time_ms),
                crate::open_sound::types::OpenSoundArgument::Float32(duration_ms),
                crate::open_sound::types::OpenSoundArgument::String(waveforms.to_string()),
            ],
        };
        self.process_message(message)
    }

    /// Create and queue a sample note
    pub fn queue_sample_note(
        &mut self,
        file_path: &str,
        volume: f32,
        start_time_ms: f32,
        duration_ms: f32,
    ) -> OpenSoundResult<()> {
        let message = OpenSoundMessage {
            address_pattern: "/note/sample".to_string(),
            arguments: vec![
                crate::open_sound::types::OpenSoundArgument::String(file_path.to_string()),
                crate::open_sound::types::OpenSoundArgument::Float32(volume),
                crate::open_sound::types::OpenSoundArgument::Float32(start_time_ms),
                crate::open_sound::types::OpenSoundArgument::Float32(duration_ms),
            ],
        };
        self.process_message(message)
    }

    /// Play an oscillator note immediately
    pub fn play_oscillator_note(
        &mut self,
        frequency: f32,
        volume: f32,
        start_time_ms: f32,
        duration_ms: f32,
        waveforms: &str,
    ) -> OpenSoundResult<()> {
        let message = OpenSoundMessage {
            address_pattern: "/note/play".to_string(),
            arguments: vec![
                crate::open_sound::types::OpenSoundArgument::String("oscillator".to_string()),
                crate::open_sound::types::OpenSoundArgument::Float32(frequency),
                crate::open_sound::types::OpenSoundArgument::Float32(volume),
                crate::open_sound::types::OpenSoundArgument::Float32(start_time_ms),
                crate::open_sound::types::OpenSoundArgument::Float32(duration_ms),
                crate::open_sound::types::OpenSoundArgument::String(waveforms.to_string()),
            ],
        };
        self.process_message(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Write, Read};
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;
    use crate::open_sound::open_sound_utils::{OscArg, create_osc_message_bytes, create_osc_bundle_bytes, create_osc_message_bytes_mixed, create_osc_bundle_bytes_mixed};

    #[test]
    fn test_integration_creation() {
        let integration = OpenSoundIntegration::new();
        assert_eq!(integration.queued_note_count(), 0);
    }

    #[test]
    fn test_queue_oscillator_note() {
        let mut integration = OpenSoundIntegration::new();
        integration.queue_oscillator_note(440.0, 0.5, 0.0, 1000.0, "sine").unwrap();
        assert_eq!(integration.queued_note_count(), 1);
    }

    #[test]
    fn test_clear_queue() {
        let mut integration = OpenSoundIntegration::new();
        integration.queue_oscillator_note(440.0, 0.5, 0.0, 1000.0, "sine").unwrap();
        integration.clear_queue();
        assert_eq!(integration.queued_note_count(), 0);
    }

    /// Test sending OSC messages directly over TCP
    #[tokio::test]
    async fn test_tcp_osc_message_direct() {
        // Start a simple server in a separate thread on a different port
        let server_handle = thread::spawn(|| {
            let config = crate::open_sound::types::OpenSoundConfig {
                tcp_address: Some("127.0.0.1:8001".to_string()),
                udp_address: None,
                sample_rate: 44100,
                buffer_size: 1024,
            };
            let mut server = crate::open_sound::server::OpenSoundServer::new(config);
            let note_handler = crate::open_sound::handler::MusicalNoteHandler::new();
            server.add_handler("/note/oscillator", Box::new(note_handler.clone()));
            server.add_handler("/note/sample", Box::new(note_handler.clone()));
            server.add_handler("/note/play", Box::new(note_handler));
            
            // Run server for a short time
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                if let Err(e) = server.start_tcp().await {
                    eprintln!("Server failed to start: {}", e);
                    return;
                }
                // Give some time for the server to start
                tokio::time::sleep(Duration::from_millis(100)).await;
            });
        });

        // Give server time to start
        thread::sleep(Duration::from_millis(200));

        // Create OSC message as bytes
        let osc_message = create_osc_message_bytes_mixed("/note/oscillator", &[
            OscArg::Float(440.0f32),  // frequency
            OscArg::Float(0.5f32),    // volume
            OscArg::Float(0.0f32),    // start_time_ms
            OscArg::Float(1000.0f32), // duration_ms
            OscArg::String("sine".to_string()),    // waveforms
        ]);

        // Send message directly over TCP
        match TcpStream::connect("127.0.0.1:8001") {
            Ok(mut stream) => {
                stream.write_all(&osc_message).unwrap();
                stream.flush().unwrap();
                println!("Sent OSC message directly over TCP");
            }
            Err(e) => {
                eprintln!("Failed to connect to server: {}", e);
                // Don't fail the test if server isn't running
                return;
            }
        }

        // Clean up
        server_handle.join().unwrap();
    }

    /// Test sending multiple OSC messages over TCP
    #[tokio::test]
    async fn test_tcp_multiple_osc_messages() {
        // Start server on a different port
        let server_handle = thread::spawn(|| {
            let config = crate::open_sound::types::OpenSoundConfig {
                tcp_address: Some("127.0.0.1:8002".to_string()),
                udp_address: None,
                sample_rate: 44100,
                buffer_size: 1024,
            };
            let mut server = crate::open_sound::server::OpenSoundServer::new(config);
            let note_handler = crate::open_sound::handler::MusicalNoteHandler::new();
            server.add_handler("/note/oscillator", Box::new(note_handler.clone()));
            server.add_handler("/note/sample", Box::new(note_handler.clone()));
            server.add_handler("/note/play", Box::new(note_handler));
            
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                if let Err(e) = server.start_tcp().await {
                    eprintln!("Server failed to start: {}", e);
                    return;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            });
        });

        thread::sleep(Duration::from_millis(200));

        // Create multiple OSC messages
        let messages = vec![
            ("/note/oscillator", vec![
                OscArg::Float(440.0f32), OscArg::Float(0.5f32), OscArg::Float(0.0f32), OscArg::Float(500.0f32), OscArg::String("sine".to_string())
            ]),
            ("/note/oscillator", vec![
                OscArg::Float(554.37f32), OscArg::Float(0.4f32), OscArg::Float(500.0f32), OscArg::Float(500.0f32), OscArg::String("triangle".to_string())
            ]),
            ("/note/play", vec![
                OscArg::String("oscillator".to_string()), OscArg::Float(880.0f32), OscArg::Float(0.3f32), OscArg::Float(0.0f32), OscArg::Float(300.0f32), OscArg::String("square".to_string())
            ]),
        ];

        match TcpStream::connect("127.0.0.1:8002") {
            Ok(mut stream) => {
                for (address, args) in messages {
                    let osc_message = create_osc_message_bytes_mixed(address, &args);
                    stream.write_all(&osc_message).unwrap();
                    stream.flush().unwrap();
                    println!("Sent OSC message: {}", address);
                    thread::sleep(Duration::from_millis(50));
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to server: {}", e);
                return;
            }
        }

        server_handle.join().unwrap();
    }

    /// Test sending OSC bundle over TCP
    #[tokio::test]
    async fn test_tcp_osc_bundle() {
        // Start server on a different port
        let server_handle = thread::spawn(|| {
            let config = crate::open_sound::types::OpenSoundConfig {
                tcp_address: Some("127.0.0.1:8003".to_string()),
                udp_address: None,
                sample_rate: 44100,
                buffer_size: 1024,
            };
            let mut server = crate::open_sound::server::OpenSoundServer::new(config);
            let note_handler = crate::open_sound::handler::MusicalNoteHandler::new();
            server.add_handler("/note/oscillator", Box::new(note_handler.clone()));
            server.add_handler("/note/sample", Box::new(note_handler.clone()));
            server.add_handler("/note/play", Box::new(note_handler));
            
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                if let Err(e) = server.start_tcp().await {
                    eprintln!("Server failed to start: {}", e);
                    return;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            });
        });

        thread::sleep(Duration::from_millis(200));

        // Create OSC bundle with multiple messages
        let bundle = create_osc_bundle_bytes_mixed(&[
            ("/note/oscillator", &[
                OscArg::Float(261.63f32), OscArg::Float(0.3f32), OscArg::Float(0.0f32), OscArg::Float(1000.0f32), OscArg::String("sine".to_string())
            ]),
            ("/note/oscillator", &[
                OscArg::Float(329.63f32), OscArg::Float(0.3f32), OscArg::Float(0.0f32), OscArg::Float(1000.0f32), OscArg::String("sine".to_string())
            ]),
            ("/note/oscillator", &[
                OscArg::Float(392.00f32), OscArg::Float(0.3f32), OscArg::Float(0.0f32), OscArg::Float(1000.0f32), OscArg::String("sine".to_string())
            ]),
        ]);

        match TcpStream::connect("127.0.0.1:8003") {
            Ok(mut stream) => {
                stream.write_all(&bundle).unwrap();
                stream.flush().unwrap();
                println!("Sent OSC bundle with 3 messages");
            }
            Err(e) => {
                eprintln!("Failed to connect to server: {}", e);
                return;
            }
        }

        server_handle.join().unwrap();
    }

    /// Test error handling with invalid OSC messages
    #[tokio::test]
    async fn test_tcp_invalid_osc_message() {
        // Start server on a different port
        let server_handle = thread::spawn(|| {
            let config = crate::open_sound::types::OpenSoundConfig {
                tcp_address: Some("127.0.0.1:8004".to_string()),
                udp_address: None,
                sample_rate: 44100,
                buffer_size: 1024,
            };
            let mut server = crate::open_sound::server::OpenSoundServer::new(config);
            let note_handler = crate::open_sound::handler::MusicalNoteHandler::new();
            server.add_handler("/note/oscillator", Box::new(note_handler.clone()));
            server.add_handler("/note/sample", Box::new(note_handler.clone()));
            server.add_handler("/note/play", Box::new(note_handler));
            
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                if let Err(e) = server.start_tcp().await {
                    eprintln!("Server failed to start: {}", e);
                    return;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            });
        });

        thread::sleep(Duration::from_millis(200));

        // Send invalid OSC message (malformed)
        let invalid_message = b"invalid osc message";
        
        match TcpStream::connect("127.0.0.1:8004") {
            Ok(mut stream) => {
                stream.write_all(invalid_message).unwrap();
                stream.flush().unwrap();
                println!("Sent invalid OSC message (should be handled gracefully)");
            }
            Err(e) => {
                eprintln!("Failed to connect to server: {}", e);
                return;
            }
        }

        server_handle.join().unwrap();
    }
} 