use crate::dsl::parser::parse_dsl;
use crate::composition::comp_utils::play_track_grid;

pub(crate) fn play() {
    println!("playing dsl 1");

    let input = r#"
let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.7 r 1.0,0.0
let delay1 = delay mix 0.7 decay 0.5 interval_ms 20.0 duration_ms 20.0 num_repeats 4 num_predelay_samples 20 num_concurrent_delays 1 
let flanger1 = flanger window_size 25 mix 0.5
let samp1 = samp:/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav:0.0005:{step}

FixedTimeNoteSequence dur Quarter tempo 20 num_steps 8 
$env1
$delay1
$flanger1

apply step:0,2,4,6 $samp1

osc:sin:880.0:0.5:0
osc:sin:880.0:0.5:1
osc:sin:880.0:0.5:2
osc:sin:880.0:0.5:3
osc:sin:880.0:0.5:4
osc:sin:880.0:0.5:5
osc:sin:880.0:0.5:6
osc:sin:880.0:0.5:7

osc:square:880.0:0.5:2
osc:square:880.0:0.5:4
osc:triangle:880.0:3.5:5
osc:triangle:880.0:3.5:7

osc:sine:440.0:0.3:3
osc:triangle:440.0:0.3:6

osc:square:880.0:0.5:4
"#;

    play_track_grid( parse_dsl(input).unwrap());
}