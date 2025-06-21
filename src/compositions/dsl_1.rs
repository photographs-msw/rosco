use crate::dsl::parser::parse_dsl;
use crate::composition::comp_utils::play_track_grid;

pub(crate) fn play() {
    println!("playing dsl 1");

    let input = r#"
    let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.9 r 1.0,0.0
    let delay1 = delay mix 0.9 decay 0.1 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2 
    let flanger1 = flanger window_size 15 mix 0.5

    FixedTimeNoteSequence dur Half tempo 60 num_steps 16
    $env1
    $delay1
    $flanger1

    osc:sine:440.0:0.9:0
    osc:triangle:440.0:0.3:0

    samp:/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav:0.005:2
    
    osc:square:880.0:9.5:4
    osc:triangle:880.0:3.5:4

    samp:/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav:0.005:6
    
    osc:sine:440.0:0.9:8
    osc:triangle:440.0:0.3:8

    samp:/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav:0.005:10
    
    osc:square:880.0:9.5:12
    osc:triangle:880.0:3.5:12
    
    samp:/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav:0.005:14
"#;

    play_track_grid(parse_dsl(input).unwrap());
}