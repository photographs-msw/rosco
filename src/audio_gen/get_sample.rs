use crate::audio_gen::oscillator;
use crate::audio_gen::oscillator::{get_gaussian_noise_sample, OscillatorTables};
use crate::audio_gen::oscillator::Waveform;
use crate::common::constants::NYQUIST_FREQUENCY;  // khz samples per second
use crate::note::playback_note::{NoteType, PlaybackNote};

pub(crate) fn get_note_sample(playback_note: &mut PlaybackNote, osc_tables: &OscillatorTables,
                              sample_position: f32, sample_count: u64) -> f32 {
    match playback_note.note_type {
        NoteType::Oscillator => {
            let mut sample = 0.0;
            for waveform in playback_note.note.waveforms.clone() {
                sample += match waveform {
                    Waveform::GaussianNoise => get_gaussian_noise_sample(),
                    Waveform::Saw => oscillator::get_sample(
                        &osc_tables.saw_table, playback_note.note.frequency, sample_count),
                    Waveform::Sine => oscillator::get_sample(
                        &osc_tables.sine_table, playback_note.note.frequency, sample_count),
                    Waveform::Square => oscillator::get_sample(
                        &osc_tables.square_table, playback_note.note.frequency, sample_count),
                    Waveform::Triangle => oscillator::get_sample(
                        &osc_tables.triangle_table, playback_note.note.frequency, sample_count),
                }
            }

            playback_note.apply_effects(playback_note.note.volume * sample, sample_position,
                                        sample_count)
        }
        NoteType::Sample => {
            let volume = playback_note.sampled_note.volume;
            let sample = playback_note.sampled_note.get_sample_at(sample_count as usize);

            playback_note.apply_effects(volume * sample, sample_position, sample_count)
        }
    }
}

pub(crate) fn get_notes_sample(playback_notes: &mut Vec<PlaybackNote>,
                               oscillator_tables: &OscillatorTables,
                               sample_position: f32, sample_count: u64) -> f32 {
    let mut out_sample = 0.0;
    for playback_note in playback_notes.iter_mut() {
        if sample_count > playback_note.playback_sample_end_time {
            continue;
        }
        out_sample += get_note_sample(playback_note, oscillator_tables, sample_position,
                                      sample_count);
    }

    if out_sample >= NYQUIST_FREQUENCY {
        out_sample = NYQUIST_FREQUENCY - 1.0;
    } else if out_sample <= -NYQUIST_FREQUENCY {
        out_sample = -NYQUIST_FREQUENCY + 1.0;
    }
    out_sample
}

