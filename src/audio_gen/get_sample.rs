use crate::audio_gen::oscillator::{get_gaussian_noise_sample, get_saw_sample, get_sin_sample,
                                   get_square_sample, get_triangle_sample};
use crate::audio_gen::oscillator::Waveform;
use crate::common::constants::{NYQUIST_FREQUENCY, SAMPLE_RATE};  // khz samples per second
use crate::note::playback_note::{NoteType, PlaybackNote};

pub(crate) fn get_note_sample(playback_note: &mut PlaybackNote, sample_clock: f32) -> f32 {
    
    match playback_note.note_type {
        NoteType::Oscillator => {
            let mut sample = 0.0;
            // TODO MOVE WAVEFORMS TO UNDERLYING NOTE
            for waveform in playback_note.waveforms.clone() {
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
                                        sample_clock / SAMPLE_RATE)
        }
        NoteType::Sample => {
            let volume = playback_note.sampled_note.volume;
            let sample = playback_note.sampled_note.next_sample();
            playback_note.apply_effects(volume * sample,
                                        sample_clock / SAMPLE_RATE)
        }
    }
}

// NOTE: Assumes playback notes of Enum Kind that include Oscillator trait
pub(crate) fn get_notes_sample(playback_notes: &mut Vec<PlaybackNote>, sample_clock: f32) -> f32 {
    let mut out_sample = 0.0;
    for playback_note in playback_notes.iter_mut() {
        let sample =
            match playback_note.note_type {
                NoteType::Oscillator => get_note_sample(playback_note, sample_clock),
                NoteType::Sample => get_note_sample(playback_note, sample_clock)
            };
        out_sample += sample;
    }

    if out_sample > NYQUIST_FREQUENCY {
        out_sample = NYQUIST_FREQUENCY;
    } else if out_sample < -NYQUIST_FREQUENCY {
        out_sample = -NYQUIST_FREQUENCY;
    } 
    out_sample
}
