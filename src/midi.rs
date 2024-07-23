use std::collections::HashMap;
// Try to use nodi, seems to have a notion of both ticks and musical meter
use nodi::midly;
use nodi::midly::num::{u28, u4, u7};

use crate::note;
use crate::sequence::{Sequence, SequenceBuilder};
use crate::utils;

static DEFAULT_BPM: u8 = 120;
static MIDI_TICKS_PER_QUARTER_NOTE: f32 = 960.0;
static SECS_PER_MIN: f32 = 60.0;
static MIDI_PITCH_TO_FREQ_HZ: Vec<f32> = (0..128)
    .map(|pitch| 440.0 * 2.0f32.powf((pitch as f32 - 69.0) / 12.0))
    .collect();

// fn get_bpm(midi: &midly::Smf) -> u8 {
//     for track in midi.tracks.iter() {
//         for event in track.iter() {
//             match event {
//                 midly::TrackEvent { delta, kind } => {
//                     match kind {
//                         midly::TrackEventKind::Meta(meta) => {
//                             match meta {
//                                 midly::MetaMessage::Tempo(tempo) => {
//                                     return (60000000 / tempo) as u8;
//                                 }
//                                 _ => {}
//                             }
//                         }
//                         _ => {}
//                     }
//                 }
//             }
//         }
//     }
//     DEFAULT_BPM
// }

fn ticks_per_millisecond(bpm: u8) -> f32 {
    ((bpm as f32 / SECS_PER_MIN) * MIDI_TICKS_PER_QUARTER_NOTE) / 1000.0
}

fn handle_note_off(channel: u4, key: u7, volume: f32, delta: u28, ticks_per_ms: u64,
                   channel_in_note_on_map: &mut HashMap<u4, bool>,
                   channel_cur_note_duration_map: &mut HashMap<u4, u28>,
                   channel_sequence_map: &mut HashMap<u4, Sequence>) {
    channel_in_note_on_map.insert(channel, false);

    // Get the ticks elapsed for all events since the last NoteOn
    // for this channel, this is the note duration
    let duration_ms =
        (delta + *channel_cur_note_duration_map.get(&channel).unwrap()).as_int() as u64 /
            ticks_per_ms;

    // Construct the Note and add it to the sequence for this channel
    let note = note::NoteBuilder::default()
        .frequency(MIDI_PITCH_TO_FREQ_HZ[key])
        .duration_ms(duration_ms)
        .volume(volume)
        .build()
        .unwrap();
    let mut channel_sequence =
        channel_sequence_map.get(&channel).unwrap();
    channel_sequence.add_note(note);
}

fn main() {

    let args = utils::get_cli_args();
    let filename = &args[0];
    let midi = midly::Smf::parse_file(filename).unwrap();

    // TODO Fix get_bpm() and get the real bmp
    // let bpm = get_bpm(&midi);
    let bpm = DEFAULT_BPM;
    let ticks_per_ms: f32 = ticks_per_millisecond(bpm);

    // TODO HANDLE START TIME FOR EACH NOTE

    let mut channel_sequence_map: HashMap<u4, Sequence> = HashMap::new();
    let mut channel_cur_note_duration_map = HashMap::new();
    let mut channel_in_note_on_map = HashMap::new();
    let mut duration: u28 = 0;
    for track in midi.tracks.iter() {
        for event in track.iter() {
            match event {
                midly::TrackEvent { delta, kind} => {
                    match kind {
                        midly::TrackEventKind::Midi { channel, message } => {
                            match message {
                                midly::MidiMessage::NoteOn { key, vel } => {
                                    if vel > 0 {
                                        channel_in_note_on_map.insert(channel, true);
                                        // If we have never seen the channel before, init the state
                                        // of the map being used to collect events into sequences
                                        if !channel_sequence_map.contains_key(&channel) {
                                            channel_sequence_map.insert(channel,
                                                                        SequenceBuilder::default()
                                                                            .build().unwrap());
                                        }
                                        // We are at the start of a new note, so reset the duration
                                        channel_cur_note_duration_map.insert(channel, 0);
                                    } else {
                                        // NoteOn with velocity of 0 is often used as a NoteOff
                                        // So capture the duration from the last non-zero NoteOn.
                                        // Otherwise, if we are NoteOn and have a velocity it is the
                                        // start of a new note in this channel so capture the tick.
                                        handle_note_off(channel,
                                                        key,
                                                        vel / 127.0,
                                                        delta,
                                                        ticks_per_ms,
                                                        &mut channel_in_note_on_map,
                                                        &mut channel_cur_note_duration_map,
                                                        &mut channel_sequence_map);
                                    }
                                }
                                midly::MidiMessage::NoteOff { key, vel } => {
                                    handle_note_off(channel,
                                                    key,
                                                    vel / 127.0,
                                                    delta,
                                                    ticks_per_ms,
                                                    &mut channel_in_note_on_map,
                                                    &mut channel_cur_note_duration_map,
                                                    &mut channel_sequence_map);
                                }
                                // If the event is not NoteOn or NoteOff, ignore it but add the
                                // ticks since the last NoteOn to the running total of the duration
                                // of the current NoteOn, if we are in a NoteOn.
                                _ => {
                                    if channel_in_note_on_map.get(&channel).unwrap() {
                                        let cur_duration: u28 =
                                            channel_cur_note_duration_map.get(channel).unwrap();
                                        channel_cur_note_duration_map.insert(channel, cur_duration + duration);
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

    // TODO HANDLE CREATING A CHANNEL FOR EACH TRACK AND ADDING ITS SEQUENCE

    // for (channel, sequence) in channel_sequence_map.iter() {
    //     let channel = channel::ChannelBuilder::default()
    //         .sequence(sequence.clone())
    //         .volume(1.0)
    //         .build()
    //         .unwrap();
    //     channels.push(channel);
    // }
    //
    // let mut index = 0;
    // loop {
    //     let mut all_channels_at_end = true;
    //     for channel in channels.iter_mut() {
    //         if !channel.sequence.at_end() {
    //             all_channels_at_end = false;
    //             let note = channel.sequence.get_note_and_advance();
    //             let duration = std::time::Duration::from_millis(note.duration_ms);
    //             println!("Playing note with frequency {} for {} ms", note.frequency, note.duration_ms);
    //             std::thread::sleep(duration);
    //         }
    //     }
    //     if all_channels_at_end {
    //         break;
    //     }
    // }
}