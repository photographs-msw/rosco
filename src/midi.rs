use std::collections::HashMap;
use nodi::midly;
use nodi::midly::num::{u28, u4, u7};

use crate::track;
use crate::note;
use crate::sequence::{Sequence, SequenceBuilder};

static DEFAULT_BPM: u8 = 120;
static MIDI_TICKS_PER_QUARTER_NOTE: f32 = 960.0;
static SECS_PER_MIN: f32 = 60.0;
static MIDI_PITCH_TO_FREQ_HZ: Vec<f32> = (0..128)
    .map(|pitch| 440.0 * 2.0f32.powf((pitch as f32 - 69.0) / 12.0))
    .collect();

pub(crate) fn midi_file_channels_into_tracks(file_name: &str) -> Vec<track::Track> {

    let mut tracks = Vec::new();
    let midi = midly::Smf::parse_file(file_name).unwrap();

    let bpm = get_bpm(&midi);
    let bpm_ticks_per_ms: u64 = ticks_per_millisecond(bpm);
    let mut track_sequence_map: HashMap<u4, Sequence> = HashMap::new();
    let mut track_cur_note_duration_map = HashMap::new();
    let mut track_cur_note_start_time_map = HashMap::new();
    let mut track_in_note_on_map = HashMap::new();
    let mut ticks_since_start: u28 = 0;
    // these two variables are just to rename the struct match type to a name matching our semantics
    let mut delta_ticks: u28 = 0;
    let mut pitch: u7 = 0;

    for track in midi.tracks.iter() {
        for event in track.iter() {
            match event {
                midly::TrackEvent { delta, kind} => {
                    delta_ticks = delta;
                    ticks_since_start += delta;

                    match kind {
                        midly::TrackEventKind::Midi { channel, message } => {

                            match message {
                                midly::MidiMessage::NoteOn { key, vel } => {
                                    pitch = key;

                                    if vel > 0 {
                                        track_in_note_on_map.insert(channel, true);
                                        // If we have never seen the channel before, init the state
                                        // of the map being used to collect events into sequences
                                        if !track_sequence_map.contains_key(&channel) {
                                            track_sequence_map.insert(channel,
                                                                        SequenceBuilder::default()
                                                                            .build().unwrap());
                                        }

                                        // We are at the start of a new note, so reset the duration
                                        // and set the start_time
                                        track_cur_note_duration_map.insert(channel, 0);
                                        track_cur_note_start_time_map
                                            .insert(channel,
                                                    ticks_since_start.as_int() as u64 / bpm_ticks_per_ms);
                                    } else {
                                        // NoteOn with velocity of 0 is the same as a NoteOff
                                        track_in_note_on_map.insert(channel, false);
                                        handle_note_off(channel,
                                                        pitch,
                                                        vel / 127.0,
                                                        track_cur_note_start_time_map
                                                            .get(&channel).unwrap(),
                                                        note_duration_ms(channel, bpm_ticks_per_ms,
                                                                         delta_ticks,
                                                                         &track_cur_note_duration_map),
                                                        &mut track_sequence_map);
                                    }
                                }

                                midly::MidiMessage::NoteOff { key, vel } => {
                                    track_in_note_on_map.insert(channel, false);
                                    handle_note_off(channel,
                                                    pitch,
                                                    vel / 127.0,
                                                    track_cur_note_start_time_map
                                                        .get(&channel).unwrap(),
                                                    note_duration_ms(channel, bpm_ticks_per_ms,
                                                                     delta_ticks,
                                                                     &track_cur_note_duration_map),
                                                    &mut track_sequence_map);
                                }

                                // If the event is not NoteOn or NoteOff, ignore it but add the
                                // ticks since the last NoteOn to the running total of the duration
                                // of the current NoteOn, if we are in a NoteOn.
                                _ => {
                                    if track_in_note_on_map.get(&channel).unwrap() {
                                        update_duration_map(channel,
                                                            delta_ticks,
                                                            &mut track_cur_note_duration_map);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    for (midi_channel, sequence) in track_sequence_map.iter() {
        let track = track::TrackBuilder::default()
            .name(format!("{}", midi_channel))
            .sequence(sequence.clone())
            .volume(1.0 / track_sequence_map.len() as f32)
            .build()
            .unwrap();
        tracks.push(track);
    }

    tracks
}

fn get_bpm(midi: &midly::Smf) -> u8 {
    for track in midi.tracks.iter() {
        for event in track.iter() {
            match event {
                midly::TrackEvent { delta, kind } => {
                    match kind {
                        midly::TrackEventKind::Meta(meta) => {
                            match meta {
                                midly::MetaMessage::Tempo(tempo) => {
                                    return (60000000 / *tempo.as_int()) as u8;
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

fn ticks_per_millisecond(bpm: u8) -> f32 {
    ((bpm as f32 / SECS_PER_MIN) * MIDI_TICKS_PER_QUARTER_NOTE) / 1000.0
}

fn update_duration_map(channel: u4,
                       delta_ticks: u28,
                       track_cur_note_duration_map: &mut HashMap<u4, u28>) {

    let cur_duration: u28 = track_cur_note_duration_map.get(*channel).unwrap();
    track_cur_note_duration_map.insert(channel, cur_duration + delta_ticks);
}

fn note_duration_ms(channel: u4,
                    bpm_ticks_per_ms: u64,
                    delta_ticks: u28,
                    track_cur_note_duration_map: &HashMap<u4, u28>) -> u64 {
    (delta_ticks + *track_cur_note_duration_map.get(&channel).unwrap()).as_int() as u64 /
        bpm_ticks_per_ms
}

fn handle_note_off(channel: u4,
                   pitch: u7,
                   volume: f32,
                   start_time_ms: u64,
                   duration_ms: u64,
                   track_sequence_map: &mut HashMap<u4, Sequence>) {

    // Construct the Note and add it to the sequence for this channel
    let note = note::NoteBuilder::default()
        .frequency(MIDI_PITCH_TO_FREQ_HZ[pitch])
        .start_time_ms(start_time_ms)
        .duration_ms(duration_ms)
        .volume(volume)
        .build()
        .unwrap();
    let mut track_sequence =
        track_sequence_map.get(&channel).unwrap();
    track_sequence.add_note(note);
}
