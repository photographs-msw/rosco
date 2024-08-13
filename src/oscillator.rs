use crate::note::Note;

pub(crate) static SAMPLE_RATE: f32 = 44100.0;
static TWO_PI: f32 = 2.0 * std::f32::consts::PI;

#[derive(Clone, Debug)]
pub(crate) enum Waveform {
    Sine,
    Triangle,
    Square,
    Saw,
}

pub(crate) fn get_waveforms(waveform_arg: &str) -> Vec<Waveform> {
    waveform_arg.split(",")
        .map( |waveform| {
            let matched = match waveform {
                "sine" => Waveform::Sine,
                "triangle" => Waveform::Triangle,
                "square" => Waveform::Square,
                "saw" => Waveform::Saw,
                _ => Waveform::Sine,
            };
            matched
        })
        .collect()
}

pub(crate) fn get_note_sample(waveforms: &Vec<Waveform>, frequency: f32, sample_clock: f32) -> f32 {
    let mut freq = 0.0;
    for waveform in waveforms {
        freq += match waveform {
            Waveform::Sine => get_sin_sample(frequency, sample_clock),
            Waveform::Triangle => get_triangle_sample(frequency, sample_clock),
            Waveform::Square => get_square_sample(frequency, sample_clock),
            Waveform::Saw => get_saw_sample(frequency, sample_clock),
        };
    }
    freq
}

pub(crate) fn get_notes_sample(notes: &Vec<Note>, channel_waveforms: &Vec<Vec<Waveform>>,
                               sample_clock: f32) -> f32 {
    let mut freq = 0.0;
    for (i, note) in notes.iter().enumerate() {
        freq += note.volume *
            get_note_sample(&channel_waveforms[i], note.frequency, sample_clock);
    }
    freq
}

fn get_sin_sample(frequency: f32, sample_clock: f32) -> f32 {
    (sample_clock * frequency * TWO_PI / SAMPLE_RATE).sin()
}

fn get_triangle_sample(frequency: f32, sample_clock: f32) -> f32 {
    4.0 * ((frequency / SAMPLE_RATE * sample_clock)
        - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
        .floor()).abs()
        - 1.0
}

fn get_square_sample(frequency: f32, sample_clock: f32) -> f32 {
    if (sample_clock * frequency / SAMPLE_RATE) % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

fn get_saw_sample(frequency: f32, sample_clock: f32) -> f32 {
    2.0 * ((frequency / SAMPLE_RATE * sample_clock)
        - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
        .floor()).abs()
        - 1.0
}
