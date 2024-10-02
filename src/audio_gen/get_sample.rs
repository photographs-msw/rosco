use crate::audio_gen::oscillator::{get_gaussian_noise_sample, get_saw_sample, get_sin_sample,
                                   get_square_sample, get_triangle_sample};
use crate::audio_gen::oscillator::Waveform;
use crate::common::constants::{NYQUIST_FREQUENCY, SAMPLE_RATE};  // khz samples per second
use crate::note::playback_note::{NoteType, PlaybackNote};

pub(crate) fn get_note_sample(playback_note: &mut PlaybackNote, sample_clock: f32,
                              sample_count: u64) -> f32 {
    
    // TEMP DEBUG
    // println!("sample_count: {}, playback_note.playback_sample_start_time {}, playback_note.playback_sample_end_time {}", 
    //          sample_count, playback_note.playback_sample_start_time, playback_note.playback_sample_end_time);
    // if sample_count < playback_note.playback_sample_start_time ||
    //     sample_count > playback_note.playback_sample_end_time {
    //     return 0.0;
    // }
    
    match playback_note.note_type {
        NoteType::Oscillator => {
            let mut sample = 0.0;
            for waveform in playback_note.note.waveforms.clone() {
                sample += match waveform {
                    Waveform::GaussianNoise => get_gaussian_noise_sample(),
                    Waveform::Saw => get_saw_sample(playback_note.note.frequency, sample_clock),
                    Waveform::Sine => get_sin_sample(playback_note.note.frequency, sample_clock),
                    Waveform::Square => get_square_sample(playback_note.note.frequency,
                                                          sample_clock),
                    Waveform::Triangle => get_triangle_sample(playback_note.note.frequency,
                                                              sample_clock),
                }
            }
            playback_note.apply_effects(playback_note.note.volume * sample,
                                        sample_clock / SAMPLE_RATE, sample_count)
        }
        NoteType::Sample => {
            let volume = playback_note.sampled_note.volume;
            let sample = playback_note.sampled_note.get_sample_at(sample_count as usize);
            
            // TEMP DEBUG
            // println!("sample: {}", sample);
            
            playback_note.apply_effects(volume * sample,
                                        sample_clock / SAMPLE_RATE, sample_count)
        }
    }
}

pub(crate) fn get_notes_sample(playback_notes: &mut Vec<PlaybackNote>, sample_clock: f32,
                               sample_count: u64) -> f32 {
    let mut out_sample = 0.0;
    for playback_note in playback_notes.iter_mut() {
        out_sample += get_note_sample(playback_note, sample_clock, sample_count);
    }

    if out_sample >= NYQUIST_FREQUENCY {
        out_sample = NYQUIST_FREQUENCY - 1.0;
    } else if out_sample <= -NYQUIST_FREQUENCY {
        out_sample = -NYQUIST_FREQUENCY + 1.0;
    } 
    out_sample
}
