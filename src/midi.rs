use std::collections::HashMap;

use nodi::midly;
use nodi::midly::num::{u28, u4, u7};

use crate::note::{Note, NoteBuilder};
use crate::sequence::{Sequence, SequenceBuilder};
use crate::track::{Track, TrackBuilder};

#[allow(dead_code)]
pub(crate) static DEFAULT_BPM: u8 = 120;
#[allow(dead_code)]
static MIDI_TICKS_PER_QUARTER_NOTE: f32 = 960.0;
static SECS_PER_MIN: f32 = 60.0;
static MIDI_PITCH_TO_FREQ_HZ: [f64; 128] = [
    0.0, 8.661957218027252, 9.177023997418988, 9.722718241315029, 10.300861153527183,
    10.913382232281373, 11.562325709738575,
    12.249857374429663, 12.978271799373287, 13.75, 14.567617547440307, 15.433853164253883,
    16.351597831287414, 17.323914436054505, 18.354047994837977, 19.445436482630058,
    20.601722307054366, 21.826764464562746, 23.12465141947715, 24.499714748859326,
    25.956543598746574, 27.5, 29.13523509488062, 30.86770632850775, 32.70319566257483,
    34.64782887210901, 36.70809598967594, 38.890872965260115, 41.20344461410875,
    43.653528929125486, 46.2493028389543, 48.999429497718666, 51.91308719749314, 55.0,
    58.27047018976124, 61.7354126570155, 65.40639132514966, 69.29565774421802, 73.41619197935188,
    77.78174593052023, 82.4068892282175, 87.30705785825097, 92.4986056779086, 97.99885899543733,
    103.82617439498628, 110.0, 116.54094037952248, 123.47082531403103, 130.8127826502993,
    138.59131548843604, 146.8323839587038, 155.56349186104046, 164.81377845643496,
    174.61411571650194, 184.9972113558172, 195.99771799087463, 207.65234878997256, 220.0,
    233.08188075904496, 246.94165062806206, 261.6255653005986, 277.1826309768721,
    293.6647679174076, 311.1269837220809, 329.6275569128699, 349.2282314330039,
    369.9944227116344, 391.99543598174927, 415.3046975799451, 440.0, 466.1637615180899,
    493.8833012561241, 523.2511306011972, 554.3652619537442, 587.3295358348151, 622.2539674441618,
    659.2551138257398, 698.4564628660078, 739.9888454232688, 783.9908719634985, 830.6093951598903,
    880.0, 932.3275230361799, 987.7666025122483, 1046.5022612023945, 1108.7305239074883,
    1174.6590716696303, 1244.5079348883237, 1318.5102276514797, 1396.9129257320155,
    1479.9776908465376, 1567.981743926997, 1661.2187903197805, 1760.0, 1864.6550460723597,
    1975.533205024496, 2093.004522404789, 2217.4610478149766, 2349.31814333926, 2489.0158697766474,
    2637.02045530296, 2793.825851464031, 2959.955381693075, 3135.9634878539946, 3322.437580639561,
    3520.0, 3729.3100921447194, 3951.066410048992, 4186.009044809578, 4434.922095629953,
    4698.63628667852, 4978.031739553295, 5274.04091060592, 5587.651702928062, 5919.91076338615,
    6271.926975707989, 6644.875161279122, 7040.0, 7458.620184289437, 7902.132820097988,
    8372.018089619156, 8869.844191259906, 9397.272573357044, 9956.06347910659,
    10548.081821211836, 11175.303405856126, 11839.8215267723, 12543.853951415975
];

// The MIDI standard doesn't support connecting NoteOn and NoteOff events, nor NoteOn events with
// > 0 velocity and NoteOn events on the same pitch with 0 velocity, which are treated as NoteOff.
// We are processing raw Midi events in a stream, so we can't do any better and can only validly
// process input which doesn't have overlapping notes on the same channel with the same pitch.
// So the natural key for a current note in NoteOn state waiting for the NoteOff is (channel, pitch)
#[derive(Debug, Eq, Hash, PartialEq)]
struct NoteKey {
    channel: u4,
    pitch: u7
}

pub(crate) fn midi_file_to_tracks(file_name: &str) -> Vec<Track> {
    let mut tracks: Vec<Track> = Vec::new();
    let data = std::fs::read(file_name).unwrap();
    let midi = midly::Smf::parse(&data).unwrap();

    // Intermediate bookkeeping of notes in process; have seen NoteOn, waiting for matching NoteOff
    let mut track_notes_map: HashMap<NoteKey, Note>= HashMap::new();
    // Output, tracks for each channel in the midi input with a sequence of notes for each track
    let mut track_sequence_map: HashMap<u4, Sequence> = HashMap::new();

    let bpm = get_bpm(&midi);
    let bpm_ticks_per_ms: f32 = ticks_per_millisecond(bpm);
    let mut ticks_since_start: u28 = u28::from(0);

    for track in midi.tracks.iter() {
        for event in track.iter() {
            match event {
                // delta is the number of ticks since the last Midi event
                midly::TrackEvent { delta, kind} => {
                    ticks_since_start += *delta;

                    match kind {
                        // channel is the MIDI channel 1..16
                        midly::TrackEventKind::Midi { channel, message } => {

                            match message {
                                // 'key' is midi pitch 1..127, vel is the velocity 1..127
                                midly::MidiMessage::NoteOn { key, vel } => {
                                    let note_key = NoteKey {channel: *channel, pitch: *key};

                                    if *vel > u7::from(0) {
                                        // If we have never seen the channel before, init the state
                                        // of the map being used to collect events into sequences
                                        if !track_sequence_map.contains_key(channel) {
                                            track_sequence_map.insert(*channel,
                                                                      SequenceBuilder::default()
                                                                          .build().unwrap());
                                        }
                                        // Update the current note for this channel.
                                        // - Capture the velocity from the NoteOn event, should
                                        //  be a value > 0 if it's a note meant to be heard
                                        // - Set the duration to 0 to start
                                        // - Capture the start time in ticks converted to msecs
                                        // Handle case of existing open note with same key by
                                        //  skipping this note if it is a duplicate
                                        if !track_notes_map.contains_key(&note_key) {
                                            track_notes_map.insert(note_key,
                                                NoteBuilder::default().
                                                    frequency(
                                                    MIDI_PITCH_TO_FREQ_HZ[key.as_int() as usize]
                                                        as f32)
                                                    .start_time_ms(
                                                        ticks_since_start.as_int() as f32 /
                                                            bpm_ticks_per_ms)
                                                    .duration_ms(0.0)
                                                    .volume(vel.as_int() as f32 / 127.0f32)
                                                    .build().unwrap()
                                            );
                                        }
                                    } else {
                                        handle_note_off(note_key,
                                                        delta,
                                                        bpm_ticks_per_ms,
                                                        &mut track_notes_map,
                                                        &mut track_sequence_map);
                                    }
                                }

                                // must capture vel to compile, but we don't use it in the block
                                #[allow(unused_variables)]
                                midly::MidiMessage::NoteOff { key, vel } => {
                                    let note_key = NoteKey {channel: *channel, pitch: *key};
                                    handle_note_off(note_key,
                                                    delta,
                                                    bpm_ticks_per_ms,
                                                    &mut track_notes_map,
                                                    &mut track_sequence_map);
                                }
                                // If the event is not NoteOn or NoteOff, ignore it but add the
                                // ticks since the last NoteOn to the running total of the duration
                                // of all current open notes for all tracks
                                _ => {
                                    for note in track_notes_map.values_mut() {
                                       note.duration_ms +=
                                            delta.as_int() as f32 / bpm_ticks_per_ms;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    for (midi_channel, sequence) in track_sequence_map.iter() {
        let track = TrackBuilder::default()
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

fn ticks_per_millisecond(bpm: u8) -> f32 {
    ((bpm as f32 / SECS_PER_MIN) * MIDI_TICKS_PER_QUARTER_NOTE) / 1000.0
}

fn handle_note_off(note_key: NoteKey,
                   delta_ticks: &u28,
                   bpm_ticks_per_ms: f32,
                   track_notes_map: &mut HashMap<NoteKey, Note>,
                   track_sequence_map: &mut HashMap<u4, Sequence>) {
    // Add the last tick delta to the note duration, copy the note to the output track sequence
    // and remove it from the current notes map
    let mut note = track_notes_map.get_mut(&note_key).unwrap().clone();
    note.duration_ms += delta_ticks.as_int() as f32 / bpm_ticks_per_ms;
    track_sequence_map.get_mut(&note_key.channel).unwrap().add_note(note);
    track_notes_map.remove(&note_key);

    // TEMP DEBUG
    println!("{:#?}\nadded to track {}", note, note_key.channel.as_int());
}
