use crate::dsl::parser::parse_dsl;
use crate::composition::comp_utils::play_track_grid;

pub(crate) fn play() {
    println!("playing dsl 1");

    let input = r#"
let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.7 r 1.0,0.0
let delay1 = delay mix 0.6 decay 0.5 interval_ms 20.0 duration_ms 80.0 num_repeats 8 num_predelay_samples 50 num_concurrent_delays 1
let flanger1 = flanger window_size 15 mix 0.35
let samp1 = samp:/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav:0.0009:{step}

FixedTimeNoteSequence dur Whole tempo 2 num_steps 1
$env1
$delay1
$flanger1
$flanger1
$flanger1

# apply step:0,2,4,6 $samp1

osc:sine:300.0:0.5:0
osc:sine:200.0:0.5:0

FixedTimeNoteSequence dur Whole tempo 4 num_steps 1
$env1
$delay1
$flanger1

# apply step:0,2,4,6 $samp1

osc:sine:400.0:0.5:0
osc:sine:267.0:0.5:0
osc:sine:400.0:0.5:1
osc:sine:267.0:0.5:1

FixedTimeNoteSequence dur Whole tempo 16 num_steps 8 
$env1
$delay1

apply step:0,1,2,3,4,5,6,7 $samp1
"#;

    play_track_grid( parse_dsl(input).unwrap());
}