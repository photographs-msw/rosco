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
mod meter;
mod dsl;

use crate::composition::computer_punk_003;

fn main() {
    computer_punk_003::play();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::parser::parse_dsl;

    #[test]
    fn test_parse_simple_script() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            a 0.1,0.8 d 0.3,0.6 s 0.8,0.4 r 1.0,0.0
            delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            osc:sine:440.0:0.5:0
            osc:square:880.0:0.3:4
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
    }

    #[test]
    fn test_parse_multiple_blocks() {
        let input = r#"
            FixedTimeNoteSequence dur Eighth tempo 140 num_steps 8
            a 0.05,0.9 d 0.2,0.7 s 0.9,0.5 r 1.0,0.0
            osc:sine:220.0:0.4:0
            
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            flanger window_size 8 mix 0.3
            samp:/path/to/sample.wav:0.6:2
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 2);
    }

    #[test]
    fn test_parse_western_pitch() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:C:0.5:0
            osc:triangle:F#:0.3:4
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_octave_western_pitch() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:4,C:0.5:0
            osc:triangle:5,F#:0.3:4
            osc:square:3,A:0.7:8
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_complex_effects() {
        let input = r#"
            FixedTimeNoteSequence dur Half tempo 100 num_steps 32
            a 0.1,0.9 d 0.4,0.6 s 0.8,0.3 r 1.0,0.0
            delay mix 0.8 decay 0.6 interval_ms 80.0 duration_ms 40.0 num_repeats 5 num_predelay_samples 15 num_concurrent_delays 3
            flanger window_size 12 mix 0.4
            lfo freq 2.5 amp 0.3 waveforms sine,triangle
            osc:sine,square:440.0:0.7:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
        assert_eq!(track.effects.flangers.len(), 1);
        assert_eq!(track.effects.lfos.len(), 1);
    }
}
