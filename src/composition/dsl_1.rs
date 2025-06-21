use crate::dsl::parser::parse_dsl;
use crate::composition::comp_utils::play_track_grid;

pub(crate) fn play() {
    println!("playing dsl 1");

    let input = r#"
    FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
    a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
    delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2

    osc:sine:440.0:0.9:0
    osc:triangle:440.0:0.3:0

    osc:square:880.0:9.5:4
    osc:triangle:880.0:3.5:4

    osc:sine:440.0:0.9:8
    osc:triangle:440.0:0.3:8

    osc:square:880.0:9.5:12
    osc:triangle:880.0:3.5:12
"#;

    play_track_grid(parse_dsl(input).unwrap());
}