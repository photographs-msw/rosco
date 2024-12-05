extern crate derive_builder;

mod audio_gen;
mod common;
mod effect;
mod envelope;
mod instrument;
mod midi;
mod note;
mod sequence;
mod track;
mod composition;

use crate::audio_gen::oscillator::Waveform;
use crate::composition::comp_utils;
use crate::effect::flanger::FlangerBuilder;
use crate::effect::lfo::LFOBuilder;
use crate::envelope::envelope::EnvelopeBuilder;
use crate::envelope::envelope_pair::EnvelopePair;
use crate::sequence::time_note_sequence::{TimeNoteSequence, TimeNoteSequenceBuilder};
use crate::track::track::Track;
use crate::track::track_grid::TrackGridBuilder;

const COMPUTER_PUNK_VERSION: &str = "001";

fn main() {
    // Init
    println!("playing 'computer punk {}'\n", COMPUTER_PUNK_VERSION);

    let waveforms = comp_utils::get_waveforms_from_arg();
    let oscillators_tables = audio_gen::oscillator::OscillatorTables::new();

    let midi_note_volume = 0.8;
    let sampled_note_volume = 0.000012;
    let sampled_note_rev_volume = 0.000042 * 0.3;
    
    // Track Effects
    // Envelopes
    let envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.25, 0.7))
        .decay(EnvelopePair(0.45, 0.8))
        .sustain(EnvelopePair(0.80, 0.7))
        .build().unwrap();
    let short_envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.03, 0.92))
        .decay(EnvelopePair(0.1, 0.87))
        .sustain(EnvelopePair(0.96, 0.85))
        .build().unwrap();

    // Flangers
    let flanger = FlangerBuilder::default()
        .window_size(17)
        .sample_buffer()
        .mix(0.18)
        .build().unwrap();
    let flanger_2 = FlangerBuilder::default()
        .window_size(6)
        .sample_buffer()
        .mix(0.15)
        .build().unwrap();

    // LFOs
    let lfo = LFOBuilder::default()
        .waveforms(vec![Waveform::Sine])
        .frequency(220.0)
        .amplitude(0.0029)
        .build().unwrap();

    // /Track Effects

    // Load Sample Notes and Tracks
    let start_time = 0.0;
    let sampled_playback_note = comp_utils::build_sampled_playback_note(
        // "/Users/markweiss/Downloads/punk_computer/001/punk_computer_003_16bit.wav",
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_008.wav",
        sampled_note_volume,
        start_time,
        vec![short_envelope],
        vec![flanger_2.clone()]
    );

    let mut sampled_playback_note_reverse = sampled_playback_note.clone();
    sampled_playback_note_reverse.sampled_note.reverse();
    sampled_playback_note_reverse.sampled_note.volume = sampled_note_rev_volume;
    sampled_playback_note_reverse.flangers = vec![flanger.clone(), flanger_2.clone()];

    let offset = 0.5;
    let mut sampled_playback_note_offset = sampled_playback_note.clone();
    sampled_playback_note_offset.sampled_note.volume = sampled_note_rev_volume;
    sampled_playback_note_offset.flangers = vec![flanger.clone(), flanger_2.clone()];
    let sampled_playback_note_offset_clone = sampled_playback_note_offset.clone();
    comp_utils::set_notes_offset(&mut vec![sampled_playback_note_offset], offset);

    let sampled_playback_note_clav = comp_utils::build_sampled_playback_note(
        // "/Users/markweiss/Downloads/punk_computer/001/punk_computer_003_16bit.wav",
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_011.wav",
        sampled_note_volume,
        start_time + 0.125,
        vec![short_envelope],
        vec![flanger_2.clone()]
    );
    let sampled_playback_note_guitar = comp_utils::build_sampled_playback_note(
        // "/Users/markweiss/Downloads/punk_computer/001/punk_computer_003_16bit.wav",
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_guitar_011.wav",
        sampled_note_volume,
        start_time + 0.0,
        vec![short_envelope],
        vec![flanger_2.clone()]
    );

    // let num_chopped_notes = 4;
    // let mut sampled_note_chopped = sampled_playback_note.clone();
    // let chopped_notes = sampled_playback_note.sampled_note
    //     .chopped(num_chopped_notes);
    // let chopped_note_duration =
    //     sampled_playback_note.note_duration_ms() / num_chopped_notes as f32;
    // let mut chopped_playback_notes: Vec<PlaybackNote> = chopped_notes.iter().enumerate()
    //     .map(|(i, note)| {
    //         let mut playback_note = sampled_playback_note.clone();
    //         playback_note.sampled_note = note.clone();
    //         playback_note.playback_sample_start_time = i as u64 * chopped_note_duration as u64;
    //         playback_note.sampled_note.start_time_ms = i as u64 as f32 * chopped_note_duration;
    //         playback_note.playback_sample_end_time = (i + 1) as u64 * chopped_note_duration as u64;
    //         playback_note.sampled_note.end_time_ms = (i + 1) as u64 as f32 * chopped_note_duration;
    //         playback_note.envelopes = vec![short_envelope.clone()];
    //         playback_note
    //     }).collect();

    let vol_factor = 2.0;
    let sample_track: Track<TimeNoteSequence> =
        comp_utils::load_note_to_new_track::<TimeNoteSequence, TimeNoteSequenceBuilder>(
            sampled_playback_note, 0.000007 * vol_factor);
    let sample_track_rev: Track<TimeNoteSequence> =
        comp_utils::load_note_to_new_track::<TimeNoteSequence, TimeNoteSequenceBuilder>(
            sampled_playback_note_reverse, 0.0000018 * vol_factor);
    let sample_track_offset: Track<TimeNoteSequence> =
        comp_utils::load_note_to_new_track::<TimeNoteSequence, TimeNoteSequenceBuilder>(
            sampled_playback_note_offset_clone, 0.000007 * vol_factor);
    let sample_track_clav: Track<TimeNoteSequence> =
        comp_utils::load_note_to_new_track::<TimeNoteSequence, TimeNoteSequenceBuilder>(
            sampled_playback_note_clav.clone(), 0.0000021 * vol_factor);
    let sample_track_guitar =
        comp_utils::load_note_to_new_track::<TimeNoteSequence, TimeNoteSequenceBuilder>(
            sampled_playback_note_guitar.clone(), 0.0000080 * vol_factor);
    // let sample_track_chopped = TrackBuilder::default()
    //     .sequence(TimeNoteSequenceBuilder::default()
    //         .sequence(vec![chopped_playback_notes])
    //         .build().unwrap())
    //     .build().unwrap();

    // Load MIDI Tracks
    let mut tracks = Vec::new();
    let mut midi_time_tracks_1: Vec<Track<TimeNoteSequence>> =
        comp_utils::load_midi_file_to_tracks::<TimeNoteSequence, TimeNoteSequenceBuilder>(
            "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001_5.mid",
            waveforms.clone(),
            vec![envelope],
            vec![flanger.clone()],
            lfo.clone(),
            midi_note_volume
        );

    for track in midi_time_tracks_1.iter_mut() {
        for playback_notes in track.sequence.notes_iter_mut() {
            comp_utils::set_notes_offset(playback_notes, 0.0);
        }
    }

    let mut midi_time_tracks_2: Vec<Track<TimeNoteSequence>> =
        comp_utils::load_midi_file_to_tracks::<TimeNoteSequence, TimeNoteSequenceBuilder>(
            "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001_reaper_2.mid",
            waveforms.clone(),
            vec![envelope],
            vec![],
            lfo.clone(),
            midi_note_volume
        );
    for track in midi_time_tracks_2.iter_mut() {
        for playback_notes in track.sequence.notes_iter_mut() {
            comp_utils::set_notes_offset(playback_notes, 3.0);
        }
    }

    // Add Sample Tracks
    tracks.append(&mut midi_time_tracks_1);
    tracks.append(&mut midi_time_tracks_2);
    tracks.push(sample_track);
    tracks.push(sample_track_offset);
    tracks.push(sample_track_clav);
    tracks.push(sample_track_guitar);
    tracks.push(sample_track_rev);
    // tracks.push(sample_track_chopped);
    let mut tracks2 = tracks.clone();


    // TEMP DEBUG
    // println!("{:#?}", midi_time_tracks_3);

    // Load and play Track Grid
    let track_grid = TrackGridBuilder::default()
        .tracks(tracks)
        .build().unwrap();

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for playback_notes in track_grid {
            tx.send(playback_notes).unwrap();
        }
    });

    println!("First loop and capture loop");
    let mut loop_playback_notes = Vec::new();
    for (i, playback_notes) in rx.iter().enumerate() {
        let mut out_notes = playback_notes.clone();

        // TEMP DEBUG
        // println!("start_time: {:#?}", playback_notes);

        if (i % 4 == 0) {
            for playback_note in out_notes.iter_mut() {
                let flanger_3 = FlangerBuilder::default()
                    .window_size(i + 1)
                    .sample_buffer()
                    .mix(0.06)
                    .build().unwrap();
                playback_note.flangers.push(flanger_3.clone());
            }
        }

        audio_gen::audio_gen::gen_notes_stream(out_notes, oscillators_tables.clone());
        loop_playback_notes.push(playback_notes);
    }

    // println!("First replay loop");

    // for _ in 0..1 {
    //     for (i, playback_notes) in loop_playback_notes.iter_mut().enumerate() {
    //         if i % 2 == 0 {
    //             for playback_note in playback_notes.iter_mut() {
    //                 let new_flanger = FlangerBuilder::default()
    //                     .window_size(11)
    //                     .sample_buffer()
    //                     .mix(0.2)
    //                     .build().unwrap();
    //                 playback_note.flangers.push(new_flanger.clone());
    //             }
    //         }
    //         audio_gen::audio_gen::gen_notes_stream(playback_notes.clone(),
    //                                                oscillators_tables.clone());
    //     }
    // }
}
