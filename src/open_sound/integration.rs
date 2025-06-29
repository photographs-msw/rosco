use std::collections::VecDeque;
use tokio::sync::mpsc;
use crate::open_sound::types::{OpenSoundMessage, OpenSoundConfig};
use crate::open_sound::error::{OpenSoundError, OpenSoundResult};
use crate::open_sound::handler::MusicalNoteHandler;
use crate::note::playback_note::PlaybackNote;
use crate::audio_gen::oscillator::OscillatorTables;
use crate::audio_gen::audio_gen::gen_notes_stream;

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
} 