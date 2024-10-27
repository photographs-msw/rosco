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

use crate::audio_gen::oscillator::{get_waveforms, Waveform};
use crate::effect::{flanger, lfo};
use crate::effect::flanger::{Flanger, FlangerBuilder};
use crate::effect::lfo::{LFO, LFOBuilder};
use crate::envelope::envelope::{Envelope, EnvelopeBuilder};
use crate::envelope::envelope_pair::EnvelopePair;
use crate::note::playback_note::{NoteType, PlaybackNote};
use crate::sequence::time_note_sequence::{TimeNoteSequence, TimeNoteSequenceBuilder};
use crate::track::track::{Track, TrackBuilder};
use crate::track::track_grid::TrackGridBuilder;

fn main() {
    // Init
    let computer_punk_version = "001";
    println!("playing 'computer punk {}'\n", computer_punk_version);

    // println!("Loading args");
    let waveforms_arg = collect_args();
    // println!("Args collected\nwaveforms: {}", waveforms_arg);
    let oscillators_tables = audio_gen::oscillator::OscillatorTables::new();//generate_sine_table();

    let midi_note_volume = 0.6;
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
    let sampled_playback_note = build_sampled_playback_note(
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
    set_notes_offset(&mut vec![sampled_playback_note_offset], offset);

    let sampled_playback_note_clav = build_sampled_playback_note(
        // "/Users/markweiss/Downloads/punk_computer/001/punk_computer_003_16bit.wav",
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_011.wav",
        sampled_note_volume,
        start_time + 0.125,
        vec![short_envelope],
        vec![flanger_2.clone()]
    );
    let sampled_playback_note_guitar = build_sampled_playback_note(
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
    let sample_track = load_sample_tracks(sampled_playback_note, 0.000007 * vol_factor);
    let sample_track_rev = load_sample_tracks(sampled_playback_note_reverse,
        0.0000018 * vol_factor);
    let sample_track_offset = load_sample_tracks(
        sampled_playback_note_offset_clone, 0.000007 * vol_factor);
    let sample_track_clav = load_sample_tracks(
        sampled_playback_note_clav.clone(), 0.0000021 * vol_factor);
    let sample_track_guitar = load_sample_tracks(
        sampled_playback_note_guitar.clone(), 0.0000080 * vol_factor);
    // let sample_track_chopped = TrackBuilder::default()
    //     .sequence(TimeNoteSequenceBuilder::default()
    //         .sequence(vec![chopped_playback_notes])
    //         .build().unwrap())
    //     .build().unwrap();

    // Load MIDI Tracks
    let waveforms =
        vec![Waveform::Sine, Waveform::Triangle, Waveform::Sine, Waveform::Saw, Waveform::Sine];//, Waveform::Triangle, Waveform::Triangle, Waveform::Sine];
    // let waveforms_2 =
    //     vec![Waveform::Sine, Waveform::Square, Waveform::Sine, Waveform::Triangle];
    // let waveforms_noise =
    //     vec![Waveform::Sine, Waveform::GaussianNoise, Waveform::Sine, Waveform::Sine];
    let mut tracks = Vec::new();
    let mut midi_time_tracks_1 = load_midi_tracks(
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001_5.mid",
        waveforms.clone(),
        vec![envelope],
        vec![flanger.clone()],
        lfo.clone(),
        midi_note_volume);
    for track in midi_time_tracks_1.iter_mut() {
        for playback_notes in track.sequence.notes_iter_mut() {
            set_notes_offset(playback_notes, 0.0);
        }
    }
    
    // let mut midi_time_tracks_2 = load_midi_tracks(
    //     "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001_reaper_2.mid",
    //     waveforms.clone(),
    //     vec![envelope],
    //     vec![],
    //     midi_note_volume);
    // for track in midi_time_tracks_2.iter_mut() {
    //     for playback_notes in track.sequence.notes_iter_mut() {
    //         set_notes_offset(playback_notes, 3.0);
    //     }
    // }
    // 
    // let mut midi_time_tracks_3 = load_midi_tracks(
    //     "/Users/markweiss/Downloads/punk_computer/001/punk_computer_002.mid",
    //     waveforms_2.clone(),
    //     vec![envelope],
    //     vec![flanger.clone(), flanger_2.clone(), flanger.clone()],
    //     midi_note_volume * 0.63);
    // for track in midi_time_tracks_3.iter_mut() {
    //     for playback_notes in track.sequence.notes_iter_mut() {
    //         set_notes_offset(playback_notes, 2.0);
    //     }
    // }
    
    // let mut midi_time_tracks_4 = load_midi_tracks(
    //     "/Users/markweiss/Downloads/punk_computer/001/punk_computer_002.mid",
    //     waveforms_2.clone(),
    //     vec![envelope],
    //     vec![flanger_2.clone()],
    //     midi_note_volume * 0.20);
    // for track in midi_time_tracks_3.iter_mut() {
    //     for playback_notes in track.sequence.notes_iter_mut() {
    //         set_notes_offset(playback_notes, 4.0);
    //     }
    // }

    // Add Sample Tracks
    tracks.append(&mut midi_time_tracks_1);
    // tracks.append(&mut midi_time_tracks_2);
    // tracks.append(&mut midi_time_tracks_3);
    // tracks.append(&mut midi_time_tracks_4);
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
        let out_notes = playback_notes.clone();
        
        // TEMP DEBUG
        // println!("start_time: {:#?}", playback_notes);
        
        // if (i % 4 == 0) {
        //     for playback_note in out_notes.iter_mut() {
        //         let flanger_3 = FlangerBuilder::default()
        //             .window_size(i + 1)
        //             .sample_buffer()
        //             .mix(0.06)
        //             .build().unwrap();
        //         playback_note.flangers.push(flanger_3.clone());
        //     }
        // }
        
        audio_gen::audio_gen::gen_notes_stream(out_notes, oscillators_tables.clone());
        loop_playback_notes.push(playback_notes);
    }
   
    println!("First replay loop");
    
    for _ in 0 .. 1 {
       for (i, playback_notes) in loop_playback_notes.iter_mut().enumerate() {
           if i % 2 == 0 {
               for playback_note in playback_notes.iter_mut() {
                   let new_flanger = FlangerBuilder::default()
                       .window_size(11)
                       .sample_buffer()
                       .mix(0.2)
                       .build().unwrap();
                   playback_note.flangers.push(new_flanger.clone());
               }
           }
           audio_gen::audio_gen::gen_notes_stream(playback_notes.clone(),
                                                  oscillators_tables.clone()); 
       }
    }
}

fn build_sampled_playback_note(file_path: &str, volume: f32, start_time: f32,
        envelopes: Vec<Envelope>, flangers: Vec<Flanger>) -> PlaybackNote {
    let sample_data=
        audio_gen::audio_gen::read_audio_file(file_path).into_boxed_slice();
    let mut sample_buf: Vec<f32> = Vec::with_capacity(note::sampled_note::BUF_STORAGE_SIZE);
    for sample in  sample_data[..].iter() {
        sample_buf.push(*sample as f32);
    }
    let mut sampled_note = note::sampled_note::SampledNoteBuilder::default()
        .volume(volume)
        .start_time_ms(start_time)
        .end_time_ms((sample_data.len() as f32 / common::constants::SAMPLE_RATE) * 1000.0)
        .build().unwrap();
    sampled_note.set_sample_buf(&sample_buf, sample_data.len());

    note::playback_note::PlaybackNoteBuilder::default()
        .note_type(NoteType::Sample)
        .sampled_note(sampled_note)
        .playback_start_time_ms(start_time)
        .playback_end_time_ms(
            start_time +
                ((sample_data.len() as f32 / common::constants::SAMPLE_RATE) * 1000.0))
        .playback_sample_start_time(start_time as u64)
        .playback_sample_end_time(sample_buf.len() as u64)
        .envelopes(envelopes)
        .flangers(flangers)
        .build().unwrap()
}

fn load_midi_tracks(file_path: &str, waveforms: Vec<Waveform>, envelopes: Vec<Envelope>,
        flangers: Vec<Flanger>, lfo: LFO, volume: f32) -> Vec<Track<TimeNoteSequence>> {
    let mut midi_time_tracks =
        midi::midi::midi_file_to_tracks::<TimeNoteSequence, TimeNoteSequenceBuilder>(
           file_path, NoteType::Oscillator);

    for (i, track) in midi_time_tracks.iter_mut().enumerate() {
        for playback_notes in track.sequence.notes_iter_mut() {
            for playback_note in playback_notes {
                playback_note.note.waveforms = waveforms.clone();
                playback_note.note.volume = volume;
                playback_note.envelopes = envelopes.clone();
                playback_note.flangers = flangers.clone();
                playback_note.lfos = vec![lfo.clone()]
            }
        }
    }

    midi_time_tracks
}

fn load_sample_tracks(mut sampled_playback_note: PlaybackNote, volume: f32) -> Track<TimeNoteSequence> {
    let mut sequence = TimeNoteSequenceBuilder::default().build().unwrap();
    sampled_playback_note.sampled_note.volume = volume;
    sequence.append_notes(&vec![sampled_playback_note.clone()]);
    TrackBuilder::default()
        .sequence(sequence)
        .build().unwrap()
}

fn set_notes_offset(playback_notes: &mut Vec<PlaybackNote>, offset: f32) {
    for playback_note in playback_notes.iter_mut() {
        playback_note.playback_start_time_ms += offset;
        playback_note.playback_end_time_ms += offset;
        playback_note.sampled_note.start_time_ms += offset;
        playback_note.sampled_note.end_time_ms += offset;
        
        if playback_note.note_type == NoteType::Oscillator{
            playback_note.note.start_time_ms += offset;
            playback_note.note.end_time_ms += offset;
        }
    }
}

fn collect_args () -> String {
    let mut waveforms_arg = String::from("sine");
    for (i, arg) in std::env::args().enumerate() {
        match i {
            // skip program name in 0th args position
            0 => continue,
            1 => waveforms_arg = arg,
            _ => break,
        }
    }

    waveforms_arg
}
