use crate::dsl::parser::parse_dsl;
use crate::composition::comp_utils::play_track_grid;

pub(crate) fn play() {
    println!("playing dsl 1");

    let input = r#"
let env1 = a 0.3,0.4 d 0.5,0.6 s 0.6,0.5 r 1.0,0.0
let env2 = a 0.1,0.9 d 0.2,0.6 s 0.8,0.6 r 1.0,0.0
let delay1 = delay mix 0.9 decay 1.0 interval_ms 30.0 duration_ms 30.0 num_repeats 3 num_predelay_samples 100 num_concurrent_delays 1
let flanger1 = flanger window_size 35 mix 0.75
let samp1 = samp:/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav:0.0006:{step}
let C3 = osc:sine,sine,sawtooth,sawtooth,sine,sine:3,C:0.5:{step} 
let CS3 = osc:sine,sine,sawtooth,sawtooth,sine,sine:3,CSharp:0.5:{step} 

FixedTimeNoteSequence dur Quarter tempo 92 num_steps 16
$env1
$flanger1
$delay1

apply step:0,2,4,6,8,10,12 $samp1

FixedTimeNoteSequence dur Quarter tempo 48 num_steps 8
$env1
$flanger1
$flanger1
$flanger1

apply step:1,3,5,7 $samp1

#FixedTimeNoteSequence dur Quarter tempo 92 num_steps 16
#$env2
#$flanger1
#
#apply step:2,6,10,14 osc:sine,sine,sawtooth,sawtooth,triangle:C:0.5:{step}
#
#FixedTimeNoteSequence dur Quarter tempo 92 num_steps 16
#$env2
#$flanger1
#
#apply step:2,6,10,14 osc:sine,sine,sawtooth,sawtooth,triangle,noise:E:0.5:{step}
#
#FixedTimeNoteSequence dur Quarter tempo 92 num_steps 16
#$env2
#$flanger1
#
#apply step:2,6,10,14 osc:sine,sine,sawtooth,sawtooth,triangle,noise:5,G:0.4:{step}
#
#FixedTimeNoteSequence dur Quarter tempo 46 num_steps 8
#$flanger1
#$flanger1
#$delay1
#
#apply step:1,3,5,7 osc:sawtooth,sawtooth:2,D:0.8:{step}
#
#FixedTimeNoteSequence dur Quarter tempo 23 num_steps 4
#$env2
#$delay1
#$delay1
#
#apply step:1,2,3,4 $samp1
"#;

    play_track_grid( parse_dsl(input).unwrap());
}