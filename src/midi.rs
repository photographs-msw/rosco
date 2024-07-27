use std::collections::HashMap;
use nodi::midly;
use nodi::midly::num::{u28, u4, u7};

use crate::track;
use crate::note;
use crate::sequence::{Sequence, SequenceBuilder};

static DEFAULT_BPM: u8 = 120;
static MIDI_TICKS_PER_QUARTER_NOTE: f32 = 960.0;
static SECS_PER_MIN: f32 = 60.0;

pub(crate) fn midi_file_channels_into_tracks(file_name: &str) -> Vec<track::Track> {

    let mut tracks = Vec::new();
    let data = std::fs::read(file_name).unwrap();
    let midi = midly::Smf::parse(&data).unwrap();

    let bpm = get_bpm(&midi);
    let bpm_ticks_per_ms: u64 = ticks_per_millisecond(bpm) as u64;
    let mut track_sequence_map: HashMap<u4, Sequence> = HashMap::new();
    let mut track_cur_note_duration_map: HashMap<&u4, u28> = HashMap::new();
    let mut track_cur_note_start_time_map = HashMap::new();
    let mut track_in_note_on_map = HashMap::new();
    let mut ticks_since_start: u28 = u28::from(0);
    // these two variables are just to rename the struct match type to a name matching our semantics
    let mut delta_ticks: u28 = u28::from(0);
    let mut pitch: u7 = u7::from(0);

    for track in midi.tracks.iter() {
        for event in track.iter() {
            match event {
                midly::TrackEvent { delta, kind} => {
                    delta_ticks = *delta;
                    ticks_since_start += *delta;

                    match kind {
                        midly::TrackEventKind::Midi { channel, message } => {

                            match message {
                                midly::MidiMessage::NoteOn { key, vel } => {
                                    pitch = *key;

                                    if *vel > u7::from(0) {
                                        track_in_note_on_map.insert(channel, true);
                                        // If we have never seen the channel before, init the state
                                        // of the map being used to collect events into sequences
                                        if !track_sequence_map.contains_key(&channel) {
                                            track_sequence_map.insert(*channel,
                                                                      SequenceBuilder::default()
                                                                          .build().unwrap());
                                        }

                                        // We are at the start of a new note, so reset the duration
                                        // and set the start_time
                                        track_cur_note_duration_map.insert(channel,
                                                                           u28::from(0));
                                        track_cur_note_start_time_map
                                            .insert(channel,
                                                    ticks_since_start.as_int() as u64 / bpm_ticks_per_ms);
                                    } else {
                                        // NoteOn with velocity of 0 is the same as a NoteOff
                                        track_in_note_on_map.insert(channel, false);
                                        handle_note_off(*channel,
                                                        pitch,
                                                        vel.as_int() as f32 / 127.0f32,
                                                        *track_cur_note_start_time_map
                                                            .get(channel).unwrap(),
                                                        note_duration_ms(*channel, bpm_ticks_per_ms,
                                                                         delta_ticks,
                                                                         &track_cur_note_duration_map),
                                                        &mut track_sequence_map);
                                    }
                                }

                                midly::MidiMessage::NoteOff { key, vel } => {
                                    track_in_note_on_map.insert(channel, false);
                                    handle_note_off(*channel,
                                                    pitch,
                                                    vel.as_int() as f32 / 127.0f32,
                                                    *track_cur_note_start_time_map
                                                        .get(&channel).unwrap(),
                                                    note_duration_ms(*channel, bpm_ticks_per_ms,
                                                                     delta_ticks,
                                                                     &track_cur_note_duration_map),
                                                    &mut track_sequence_map);
                                }

                                // If the event is not NoteOn or NoteOff, ignore it but add the
                                // ticks since the last NoteOn to the running total of the duration
                                // of the current NoteOn, if we are in a NoteOn.
                                _ => {
                                    if *track_in_note_on_map.get(&channel).unwrap() {
                                        let cur_duration: u28 =
                                            *track_cur_note_duration_map.get(&channel).unwrap();
                                        track_cur_note_duration_map
                                            .insert(channel, cur_duration + delta_ticks);
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
                                    return (60000000 / (*tempo).as_int()) as u8;
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

fn note_duration_ms(channel: u4,
                    bpm_ticks_per_ms: u64,
                    delta_ticks: u28,
                    track_cur_note_duration_map: &HashMap<&u4, u28>) -> u64 {
    (delta_ticks + *track_cur_note_duration_map.get(&channel).unwrap()).as_int() as u64 /
        bpm_ticks_per_ms
}

fn handle_note_off(channel: u4,
                   pitch: u7,
                   volume: f32,
                   start_time_ms: u64,
                   duration_ms: u64,
                   track_sequence_map: &mut HashMap<u4, Sequence>) {

    // TODO NEED A CRATE staticvec
    let midi_pitch_to_freq_hz: [f64; 128] = [
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

    // Construct the Note and add it to the sequence for this channel
    let note = note::NoteBuilder::default()
        .frequency(midi_pitch_to_freq_hz[pitch.as_int() as usize] as f32)
        .start_time_ms(start_time_ms)
        .duration_ms(duration_ms)
        .volume(volume)
        .build()
        .unwrap();
    track_sequence_map.get_mut(&channel).unwrap().add_note(note);
}
