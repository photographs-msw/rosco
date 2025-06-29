use std::collections::HashMap;
use crate::open_sound::types::{OpenSoundMessage, OpenSoundArgument};
use crate::open_sound::error::{OpenSoundError, OpenSoundResult};
use crate::note::playback_note::{PlaybackNote, NoteType, PlaybackNoteBuilder};
use crate::note::note::{Note, NoteBuilder};
use crate::note::sampled_note::{SampledNote, SampledNoteBuilder};
use crate::audio_gen::oscillator::Waveform;
use crate::audio_gen::audio_gen::gen_notes_stream;
use crate::audio_gen::oscillator::OscillatorTables;

/// Trait for handling OSC route patterns
pub trait RouteHandler: Send + Sync {
    fn handle(&self, message: OpenSoundMessage) -> OpenSoundResult<()>;
}

/// Handler for musical note messages
#[derive(Clone)]
pub struct MusicalNoteHandler {
    oscillator_tables: OscillatorTables,
}

impl MusicalNoteHandler {
    pub fn new() -> Self {
        Self {
            oscillator_tables: OscillatorTables::new(),
        }
    }

    /// Handle oscillator note messages
    /// Route: /note/oscillator
    /// Arguments: frequency, volume, start_time_ms, duration_ms, waveforms
    pub fn handle_oscillator_note(&self, message: OpenSoundMessage) -> OpenSoundResult<PlaybackNote> {
        if message.arguments.len() < 5 {
            return Err(OpenSoundError::InvalidArgumentCount {
                expected: 5,
                actual: message.arguments.len(),
            });
        }

        let frequency = message.arguments[0].as_f32()
            .ok_or_else(|| OpenSoundError::InvalidArgumentType {
                expected: "f32".to_string(),
                actual: format!("{:?}", message.arguments[0]),
            })?;

        let volume = message.arguments[1].as_f32()
            .ok_or_else(|| OpenSoundError::InvalidArgumentType {
                expected: "f32".to_string(),
                actual: format!("{:?}", message.arguments[1]),
            })?;

        let start_time_ms = message.arguments[2].as_f32()
            .ok_or_else(|| OpenSoundError::InvalidArgumentType {
                expected: "f32".to_string(),
                actual: format!("{:?}", message.arguments[2]),
            })?;

        let duration_ms = message.arguments[3].as_f32()
            .ok_or_else(|| OpenSoundError::InvalidArgumentType {
                expected: "f32".to_string(),
                actual: format!("{:?}", message.arguments[3]),
            })?;

        let waveforms_str = message.arguments[4].as_string()
            .ok_or_else(|| OpenSoundError::InvalidArgumentType {
                expected: "string".to_string(),
                actual: format!("{:?}", message.arguments[4]),
            })?;

        let waveforms = self.parse_waveforms(&waveforms_str)?;
        let end_time_ms = start_time_ms + duration_ms;

        let note = NoteBuilder::default()
            .frequency(frequency)
            .volume(volume)
            .start_time_ms(start_time_ms)
            .end_time_ms(end_time_ms)
            .waveforms(waveforms)
            .build()
            .map_err(|e| OpenSoundError::AudioError(format!("Failed to build note: {:?}", e)))?;

        let playback_note = PlaybackNoteBuilder::default()
            .note_type(NoteType::Oscillator)
            .note(note)
            .playback_start_time_ms(start_time_ms)
            .playback_end_time_ms(end_time_ms)
            .build()
            .map_err(|e| OpenSoundError::AudioError(format!("Failed to build playback note: {:?}", e)))?;

        Ok(playback_note)
    }

    /// Handle sample note messages
    /// Route: /note/sample
    /// Arguments: file_path, volume, start_time_ms, duration_ms
    pub fn handle_sample_note(&self, message: OpenSoundMessage) -> OpenSoundResult<PlaybackNote> {
        if message.arguments.len() < 4 {
            return Err(OpenSoundError::InvalidArgumentCount {
                expected: 4,
                actual: message.arguments.len(),
            });
        }

        let file_path = message.arguments[0].as_string()
            .ok_or_else(|| OpenSoundError::InvalidArgumentType {
                expected: "string".to_string(),
                actual: format!("{:?}", message.arguments[0]),
            })?;

        let volume = message.arguments[1].as_f32()
            .ok_or_else(|| OpenSoundError::InvalidArgumentType {
                expected: "f32".to_string(),
                actual: format!("{:?}", message.arguments[1]),
            })?;

        let start_time_ms = message.arguments[2].as_f32()
            .ok_or_else(|| OpenSoundError::InvalidArgumentType {
                expected: "f32".to_string(),
                actual: format!("{:?}", message.arguments[2]),
            })?;

        let duration_ms = message.arguments[3].as_f32()
            .ok_or_else(|| OpenSoundError::InvalidArgumentType {
                expected: "f32".to_string(),
                actual: format!("{:?}", message.arguments[3]),
            })?;

        let end_time_ms = start_time_ms + duration_ms;

        let sampled_note = SampledNoteBuilder::default()
            .file_path(file_path)
            .volume(volume)
            .start_time_ms(start_time_ms)
            .end_time_ms(end_time_ms)
            .build()
            .map_err(|e| OpenSoundError::AudioError(format!("Failed to build sampled note: {:?}", e)))?;

        let playback_note = PlaybackNoteBuilder::default()
            .note_type(NoteType::Sample)
            .sampled_note(sampled_note)
            .playback_start_time_ms(start_time_ms)
            .playback_end_time_ms(end_time_ms)
            .build()
            .map_err(|e| OpenSoundError::AudioError(format!("Failed to build playback note: {:?}", e)))?;

        Ok(playback_note)
    }

    /// Handle immediate play messages
    /// Route: /note/play
    /// Arguments: note_type, ... (other note parameters)
    pub fn handle_play_note(&self, message: OpenSoundMessage) -> OpenSoundResult<()> {
        if message.arguments.is_empty() {
            return Err(OpenSoundError::MissingArgument("note_type".to_string()));
        }

        let note_type = message.arguments[0].as_string()
            .ok_or_else(|| OpenSoundError::InvalidArgumentType {
                expected: "string".to_string(),
                actual: format!("{:?}", message.arguments[0]),
            })?;

        let playback_note = match note_type.as_str() {
            "oscillator" => {
                // Create a new message with the remaining arguments
                let mut osc_message = message.clone();
                osc_message.arguments.remove(0); // Remove note_type
                osc_message.address_pattern = "/note/oscillator".to_string();
                self.handle_oscillator_note(osc_message)?
            }
            "sample" => {
                // Create a new message with the remaining arguments
                let mut samp_message = message.clone();
                samp_message.arguments.remove(0); // Remove note_type
                samp_message.address_pattern = "/note/sample".to_string();
                self.handle_sample_note(samp_message)?
            }
            _ => return Err(OpenSoundError::UnsupportedMessageType(note_type)),
        };

        // Play the note immediately
        gen_notes_stream(vec![playback_note], self.oscillator_tables.clone());
        Ok(())
    }

    /// Parse waveform string into Waveform enum
    fn parse_waveforms(&self, waveforms_str: &str) -> OpenSoundResult<Vec<Waveform>> {
        let mut waveforms = Vec::new();
        for waveform_str in waveforms_str.split(',') {
            let waveform = match waveform_str.trim().to_lowercase().as_str() {
                "sine" | "sin" => Waveform::Sine,
                "square" | "sqr" => Waveform::Square,
                "triangle" | "tri" => Waveform::Triangle,
                "sawtooth" | "saw" => Waveform::Saw,
                "gaussiannoise" | "noise" => Waveform::GaussianNoise,
                _ => return Err(OpenSoundError::InvalidArgumentType {
                    expected: "valid waveform".to_string(),
                    actual: waveform_str.to_string(),
                }),
            };
            waveforms.push(waveform);
        }
        Ok(waveforms)
    }
}

impl RouteHandler for MusicalNoteHandler {
    fn handle(&self, message: OpenSoundMessage) -> OpenSoundResult<()> {
        match message.address_pattern.as_str() {
            "/note/oscillator" => {
                let _playback_note = self.handle_oscillator_note(message)?;
                Ok(())
            }
            "/note/sample" => {
                let _playback_note = self.handle_sample_note(message)?;
                Ok(())
            }
            "/note/play" => {
                self.handle_play_note(message)
            }
            _ => Err(OpenSoundError::InvalidAddressPattern(message.address_pattern)),
        }
    }
}

/// Handler for managing multiple routes
pub struct RouteManager {
    handlers: HashMap<String, Box<dyn RouteHandler>>,
}

impl RouteManager {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn add_handler(&mut self, pattern: &str, handler: Box<dyn RouteHandler>) {
        self.handlers.insert(pattern.to_string(), handler);
    }

    pub fn handle_message(&self, message: OpenSoundMessage) -> OpenSoundResult<()> {
        // Try exact match first
        if let Some(handler) = self.handlers.get(&message.address_pattern) {
            return handler.handle(message);
        }

        // Try pattern matching (simple wildcard support)
        for (pattern, handler) in &self.handlers {
            if self.matches_pattern(pattern, &message.address_pattern) {
                return handler.handle(message);
            }
        }

        Err(OpenSoundError::InvalidAddressPattern(message.address_pattern))
    }

    fn matches_pattern(&self, pattern: &str, address: &str) -> bool {
        // Simple wildcard matching - can be extended for more complex patterns
        if pattern.ends_with("/*") {
            let pattern_prefix = &pattern[..pattern.len() - 2];
            address.starts_with(pattern_prefix)
        } else {
            pattern == address
        }
    }
} 