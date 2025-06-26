use crate::dsl::parser::parse_dsl;
use crate::composition::comp_utils::play_track_grid;

pub(crate) fn play() {
    println!("playing dsl 1");

    let input = r#"
let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.7 r 1.0,0.0
let delay1 = delay mix 0.7 decay 0.5 interval_ms 50.0 duration_ms 30.0 num_repeats 8 num_predelay_samples 10 num_concurrent_delays 2 
let flanger1 = flanger window_size 25 mix 0.5
let samp1 = samp:/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav:0.005:{step}

FixedTimeNoteSequence dur Whole tempo 120 num_steps 16
$env1
$delay1
$flanger1

osc:sine:440.0:0.9:0

apply step:0,2 $samp1

"#;

    play_track_grid(parse_dsl(input).unwrap());
}