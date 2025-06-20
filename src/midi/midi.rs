use std::collections::HashMap;

use nodi::midly;
use nodi::midly::num::{u28, u4, u7, u15};

use crate::note::constants;
use crate::note::note::NoteBuilder;
use crate::note::playback_note::{NoteType, PlaybackNote, PlaybackNoteBuilder};
use crate::note::sampled_note::SampledNoteBuilder;
use crate::sequence::note_sequence_trait::{AppendNote, BuilderWrapper};
use crate::track::track::{Track, TrackBuilder};

#[allow(dead_code)]
pub(crate) static DEFAULT_BPM: u8 = 120;
#[allow(dead_code)]
static MSECS_PER_MIN: f32 = 60000.0;

// The MIDI standard doesn't support connecting NoteOn and NoteOff events, nor NoteOn events with
// > 0 velocity and NoteOn events on the same pitch with 0 velocity, which are treated as NoteOff.
// We are processing raw Midi events in a stream, so we can't do any better and can only validly
// process input which doesn't have overlapping notes on the same channel with the same pitch.
// So the natural key for a current note in NoteOn state waiting for the NoteOff is (channel, pitch)
/// Trait for sequences that can provide notes at a specific index
pub(crate) trait HasGetNotesAt {
    fn get_notes_at(&self, index: usize) -> Vec<PlaybackNote>;
}

/// Trait for sequences that can provide their length
pub(crate) trait HasSequenceLen {
    fn sequence_len(&self) -> usize;
}

#[allow(dead_code)]
#[derive(Debug, Eq, Hash, PartialEq)]
struct NoteKey {
    channel: u4,
    pitch: u7
}

pub(crate) fn midi_file_to_tracks<
    SequenceType: AppendNote + Clone,
    SequenceBuilderType: BuilderWrapper<SequenceType>
>
(file_name: &str, note_type: NoteType) -> Vec<Track<SequenceType>> {

    let mut tracks: Vec<Track<SequenceType>> = Vec::new();
    let data = std::fs::read(file_name).unwrap();
    let midi = midly::Smf::parse(&data).unwrap();

    // Map key is channel and pitch, so there can be more tha one notes in process on at channel
    //  but only one per pitch. This is of course a bug / limitation.
    let mut track_notes_map: HashMap<NoteKey, PlaybackNote>= HashMap::new();
    let mut track_sequence_map: HashMap<u4, SequenceType> = HashMap::new();

    let bpm = get_beats_per_minute(&midi);
    let ticks_per_beat = get_ticks_per_beat(&midi);
    let ticks_per_ms: f32 = get_ticks_per_ms(ticks_per_beat, bpm);
    let mut ticks_since_start: u28 = u28::from(0);
    for track in midi.tracks.iter() {
        for event in track.iter() {
            match event {
                // delta is the number of ticks since the last Midi event
                midly::TrackEvent { delta, kind} => {
                    ticks_since_start += *delta;

                    match kind {
                        midly::TrackEventKind::Midi { channel, message } => {
                            match message {
                                // 'key' is midi pitch 1..127
                                midly::MidiMessage::NoteOn { key, vel } => {
                                    let note_key = NoteKey {channel: *channel, pitch: *key};

                                    if *vel > u7::from(0) {
                                        // If we have never seen the channel before, init the state
                                        // of the map being used to collect events into sequences
                                        if !track_sequence_map.contains_key(channel) {
                                            track_sequence_map.insert(*channel,
                                                                      SequenceBuilderType::new());
                                        }
                                        // Update the current note for this channel.
                                        // - Capture the velocity from the NoteOn event, should
                                        //  be a value > 0 if it's a note meant to be heard
                                        // - Set the duration to 0 to start
                                        // - Capture the start time in ticks converted to msecs

                                        // TODO THIS SHOULD CLOSE THE OPEN NOTE AND OPEN A NEW ONE
                                        // Handle case of existing open note with same key by
                                        //  skipping this note if it is a duplicate
                                        if !track_notes_map.contains_key(&note_key) {
                                            let note_start_time_ms =
                                                ticks_since_start.as_int() as f32 / ticks_per_ms;
                                            match note_type {
                                                NoteType::Oscillator => { 
                                                    let note =
                                                        NoteBuilder::default().
                                                            frequency(
                                                                constants::PITCH_TO_FREQ_HZ[key.as_int() as usize] as f32)
                                                            .volume(vel.as_int() as f32 / 127.0f32)
                                                            .start_time_ms(note_start_time_ms)
                                                            .end_time_ms(note_start_time_ms)
                                                            .build().unwrap();

                                                    track_notes_map.insert(
                                                       note_key,
                                                       PlaybackNoteBuilder::default()
                                                           .note_type(note_type)
                                                           .note(note)
                                                           .playback_start_time_ms(note_start_time_ms)
                                                           .playback_end_time_ms(note_start_time_ms)
                                                           .build().unwrap());
                                                }
                                                NoteType::Sample => {
                                                    let sampled_note =
                                                        SampledNoteBuilder::default()
                                                            .volume(vel.as_int() as f32 / 127.0f32)
                                                            .start_time_ms(note_start_time_ms)
                                                            .end_time_ms(note_start_time_ms)
                                                            .build().unwrap();
                                                    track_notes_map.insert(
                                                        note_key,
                                                        PlaybackNoteBuilder::default()
                                                            .note_type(note_type)
                                                            .sampled_note(sampled_note)
                                                            .playback_start_time_ms(note_start_time_ms)
                                                            .playback_end_time_ms(note_start_time_ms)
                                                            .build().unwrap());
                                                    
                                                }
                                            }
                                        }
                                        // 0 volume for a note we got the start of previously
                                    } else if track_sequence_map.contains_key(channel) &&
                                        track_notes_map.contains_key(&note_key) {
                                        let ms_since_start =
                                            ticks_since_start.as_int() as f32 / ticks_per_ms;
                                        handle_note_off(note_key,
                                                        ms_since_start,
                                                        &mut track_notes_map,
                                                        &mut track_sequence_map);
                                    }
                                }

                                #[allow(unused_variables)]
                                midly::MidiMessage::NoteOff { key, vel } => {
                                    let note_key = NoteKey {channel: *channel, pitch: *key};
                                    let ms_since_start =
                                        ticks_since_start.as_int() as f32 / ticks_per_ms;
                                    handle_note_off(note_key,
                                                    ms_since_start,
                                                    &mut track_notes_map,
                                                    &mut track_sequence_map);
                                }

                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    for (midi_channel, sequence) in track_sequence_map.iter() {
        let track= TrackBuilder::default()
            .num(midi_channel.as_int() as i16)
            .sequence(sequence.clone())
            .volume(1.0 / track_sequence_map.len() as f32)
            .build()
            .unwrap();
        tracks.push(track);
    }

    tracks
}

#[allow(dead_code)]
pub(crate) fn get_beats_per_minute(midi: &midly::Smf) -> u8 {
    for track in midi.tracks.iter() {
        for event in track.iter() {
            match event {
                #[allow(unused_variables)]
                midly::TrackEvent { delta, kind } => {
                    match kind {
                        midly::TrackEventKind::Meta(meta) => {
                            match meta {
                                midly::MetaMessage::Tempo(tempo) => {
                                    let normalized_tempo=
                                        if tempo.as_int() > 0 {tempo.as_int()} else {1};
                                    return (60000000 / normalized_tempo) as u8;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    DEFAULT_BPM
}

#[allow(dead_code)]
pub(crate) fn get_ticks_per_beat(midi: &midly::Smf) -> u15 {
    let header = midi.header;

    match header.timing {
        midly::Timing::Metrical(ticks_per_beat) => {
            return ticks_per_beat;
        },
        _ => {
            panic!("Only Metrical timing is supported");
        }
    }
}

#[allow(dead_code)]
pub(crate) fn get_ticks_per_ms(ticks_per_beat: u15, beats_per_minute: u8) -> f32 {
    (ticks_per_beat.as_int() as f32 * beats_per_minute as f32) / MSECS_PER_MIN
}

#[allow(dead_code)]
fn handle_note_off<SequenceType: AppendNote>(note_key: NoteKey,
                                             ms_since_start: f32,
                                             track_notes_map: &mut HashMap<NoteKey, PlaybackNote>,
                                             track_sequence_map: &mut HashMap<u4, SequenceType>) {
    // Add the last tick delta to the note duration, copy the note to the output track sequence
    // and remove it from the current notes map
    let mut playback_note = track_notes_map.get_mut(&note_key).unwrap().clone();
    playback_note.set_note_end_time_ms(ms_since_start);
    playback_note.playback_end_time_ms = ms_since_start;
    track_sequence_map.get_mut(&note_key.channel).unwrap().append_note(playback_note);
    track_notes_map.remove(&note_key);
}

/// Converts a vector of tracks to a MIDI file and writes it to disk
/// Each track becomes a separate MIDI track in the output file
pub(crate) fn tracks_to_midi_file<SequenceType>(
    tracks: Vec<Track<SequenceType>>,
    file_name: &str,
    bpm: u8
) where
    SequenceType: HasGetNotesAt + HasSequenceLen,
{
    let mut midi_tracks: Vec<Vec<midly::TrackEvent>> = Vec::new();

    // Create header track with tempo information
    let mut header_track = Vec::new();

    // Add tempo meta event at the beginning
    let tempo_value = 60000000 / bpm as u32; // Convert BPM to microseconds per quarter note
    header_track.push(midly::TrackEvent {
        delta: u28::from(0),
        kind: midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(
            midly::num::u24::from(tempo_value)
        )),
    });

    // Add end of track
    header_track.push(midly::TrackEvent {
        delta: u28::from(0),
        kind: midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
    });

    midi_tracks.push(header_track);

    // Convert each track to MIDI
    for (track_index, track) in tracks.iter().enumerate() {
        let channel = u4::from((track_index as u8).min(15)); // MIDI has 16 channels (0-15)
        let midi_track = track_to_midi_track(track, channel, bpm);
        midi_tracks.push(midi_track);
    }

    // Create MIDI file structure
    let ticks_per_beat = u15::from(480); // Standard MIDI resolution
    let header = midly::Header {
        format: midly::Format::Parallel, // Type 1 MIDI file (multiple tracks)
        timing: midly::Timing::Metrical(ticks_per_beat),
    };

    let smf = midly::Smf {
        header,
        tracks: midi_tracks,
    };

    // Write to file
    let mut data = Vec::new();
    smf.write(&mut data).unwrap();
    std::fs::write(file_name, data).unwrap();
}

/// Converts a single track to a MIDI track
fn track_to_midi_track<SequenceType>(
    track: &Track<SequenceType>,
    channel: u4,
    bpm: u8
) -> Vec<midly::TrackEvent>
where
    SequenceType: HasGetNotesAt + HasSequenceLen,
{
    let mut midi_events: Vec<midly::TrackEvent> = Vec::new();
    let mut current_time_ticks: u32 = 0;

    // Calculate ticks per millisecond
    let ticks_per_ms = get_ticks_per_ms(u15::from(480), bpm);

    // Collect all notes from the sequence with their timing
    let mut all_notes: Vec<(f32, f32, PlaybackNote)> = Vec::new();

    // Iterate through all positions in the sequence
    let sequence_length = track.sequence.sequence_len();
    for index in 0..sequence_length {
        let notes_at_index = track.sequence.get_notes_at(index);
        for note in notes_at_index {
            let start_time = note.note_start_time_ms();
            let end_time = note.note_end_time_ms();
            all_notes.push((start_time, end_time, note));
        }
    }

    // Sort notes by start time
    all_notes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Convert notes to MIDI events
    for (start_time_ms, end_time_ms, playback_note) in all_notes {
        // Calculate MIDI pitch and velocity
        let (pitch, velocity) = note_to_midi_params(&playback_note, track.volume);

        // Calculate timing in ticks
        let start_ticks = (start_time_ms * ticks_per_ms) as u32;
        let end_ticks = (end_time_ms * ticks_per_ms) as u32;

        // Note On event
        let note_on_delta = if start_ticks >= current_time_ticks {
            start_ticks - current_time_ticks
        } else {
            0
        };

        midi_events.push(midly::TrackEvent {
            delta: u28::from(note_on_delta),
            kind: midly::TrackEventKind::Midi {
                channel,
                message: midly::MidiMessage::NoteOn {
                    key: pitch,
                    vel: velocity,
                },
            },
        });

        current_time_ticks = start_ticks;

        // Note Off event
        let note_off_delta = end_ticks - current_time_ticks;

        midi_events.push(midly::TrackEvent {
            delta: u28::from(note_off_delta),
            kind: midly::TrackEventKind::Midi {
                channel,
                message: midly::MidiMessage::NoteOff {
                    key: pitch,
                    vel: u7::from(64), // Standard note off velocity
                },
            },
        });

        current_time_ticks = end_ticks;
    }

    // Add end of track
    midi_events.push(midly::TrackEvent {
        delta: u28::from(0),
        kind: midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
    });

    midi_events
}



/// Converts a PlaybackNote to MIDI pitch and velocity
fn note_to_midi_params(playback_note: &PlaybackNote, track_volume: f32) -> (u7, u7) {
    let pitch = match playback_note.note_type {
        NoteType::Oscillator => {
            // Convert frequency to MIDI pitch
            frequency_to_midi_pitch(playback_note.note.frequency)
        }
        NoteType::Sample => {
            // For samples, we might use a default pitch or extract it from metadata
            // For now, use middle C (60) as default
            u7::from(60)
        }
    };

    // Calculate velocity from note volume and track volume
    let combined_volume = playback_note.note_volume() * track_volume;
    let velocity = (combined_volume * 127.0).clamp(0.0, 127.0) as u8;

    (pitch, u7::from(velocity))
}

/// Converts frequency in Hz to MIDI pitch number
fn frequency_to_midi_pitch(frequency: f32) -> u7 {
    // Find the closest match in the PITCH_TO_FREQ_HZ array
    let mut closest_pitch = 0;
    let mut min_diff = f32::INFINITY;

    for (pitch, &freq_hz) in constants::PITCH_TO_FREQ_HZ.iter().enumerate() {
        let diff = (frequency - freq_hz as f32).abs();
        if diff < min_diff {
            min_diff = diff;
            closest_pitch = pitch;
        }
    }

    // Clamp to valid MIDI range (0-127)
    u7::from((closest_pitch as u8).clamp(0, 127))
}

#[cfg(test)]
mod test_tracks_to_midi {
    use super::*;
    use crate::note::note::NoteBuilder;
    use crate::note::playback_note::{PlaybackNoteBuilder, NoteType};
    use crate::sequence::time_note_sequence::TimeNoteSequenceBuilder;
    use crate::track::track::TrackBuilder;
    use crate::track::track_effects::TrackEffectsBuilder;

    fn setup_test_note(frequency: f32, start_time: f32, end_time: f32, volume: f32) -> PlaybackNote {
        PlaybackNoteBuilder::default()
            .note_type(NoteType::Oscillator)
            .note(
                NoteBuilder::default()
                    .frequency(frequency)
                    .start_time_ms(start_time)
                    .end_time_ms(end_time)
                    .volume(volume)
                    .build()
                    .unwrap()
            )
            .playback_start_time_ms(start_time)
            .playback_end_time_ms(end_time)
            .build()
            .unwrap()
    }

    #[test]
    fn test_frequency_to_midi_pitch() {
        // Test A4 = 440 Hz should map to MIDI note 69
        let pitch = frequency_to_midi_pitch(440.0);
        assert_eq!(pitch.as_int(), 69);

        // Test middle C = ~261.63 Hz should map to MIDI note 60
        let pitch = frequency_to_midi_pitch(261.63);
        assert_eq!(pitch.as_int(), 60);
    }

    #[test]
    fn test_note_to_midi_params() {
        let note = setup_test_note(440.0, 0.0, 1000.0, 0.8);
        let track_volume = 1.0;

        let (pitch, velocity) = note_to_midi_params(&note, track_volume);

        assert_eq!(pitch.as_int(), 69); // A4
        assert_eq!(velocity.as_int(), (0.8 * 127.0) as u8); // 80% volume
    }

    #[test]
    fn test_tracks_to_midi_file_basic() {
        // Create a simple track with a few notes
        let mut sequence = TimeNoteSequenceBuilder::default().build().unwrap();

        // Add some test notes
        let notes = vec![
            setup_test_note(261.63, 0.0, 500.0, 0.8),    // Middle C
            setup_test_note(293.66, 500.0, 1000.0, 0.7), // D
            setup_test_note(329.63, 1000.0, 1500.0, 0.9), // E
        ];

        sequence.append_notes(&notes);

        let track = TrackBuilder::default()
            .num(1)
            .sequence(sequence)
            .volume(0.8)
            .effects(TrackEffectsBuilder::default().build().unwrap())
            .build()
            .unwrap();

        let tracks = vec![track];

        // Test that the function doesn't panic
        tracks_to_midi_file(tracks, "test_output.mid", 120);

        // Clean up test file
        let _ = std::fs::remove_file("test_output.mid");
    }
}
