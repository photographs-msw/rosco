use crate::audio_gen::audio_gen::gen_notes_stream;
use crate::audio_gen::oscillator::OscillatorTables;
use crate::dsl::parser::parse_dsl;
use crate::composition::comp_utils::play_track_grid;

pub(crate) fn play() {
    println!("playing dsl 1");

    let input = r#"
let samp1 = samp:/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav:0.005:{step}

FixedTimeNoteSequence dur Half tempo 40 num_steps 8 

apply step:0,2 $samp1

osc:square:880.0:0.5:4
osc:triangle:880.0:3.5:4
osc:triangle:880.0:3.5:6

osc:sine:440.0:0.4:7
osc:triangle:440.0:0.3:7

osc:square:880.0:0.5:4
"#;

    let track_grid = parse_dsl(input).unwrap();

    println!("track grid");

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for playback_notes in track_grid {
            if tx.send(playback_notes).is_err() {
                // The receiver has hung up, so we can stop the thread.
                break;
            }
        }
        println!("tx exiting");
    });
    
    for playback_notes in rx.iter() {
        gen_notes_stream(playback_notes, OscillatorTables::new());
    }
    println!("rx complete");
    
}