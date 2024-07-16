extern crate cpal;

pub(crate) static SAMPLE_RATE: f32 = 44100.0;
static TWO_PI: f32 = 2.0 * std::f32::consts::PI;

#[derive(Clone, Copy)]
pub(crate) enum OscType {
    Sine,
    Triangle,
    Square,
    Saw,
}

pub(crate) fn get_osc_types(osc_type_arg: &str) -> Vec<OscType> {
    let mut osc_types: Vec<OscType> = Vec::new();
    let osc_type_args = osc_type_arg.split(",");
    for osc_type_arg in osc_type_args {
        let osc_type: OscType = match osc_type_arg {
            "sine" => OscType::Sine,
            "triangle" => OscType::Triangle,
            "square" => OscType::Square,
            "saw" => OscType::Saw,
            _ => OscType::Sine,
        };
        osc_types.push(osc_type);
    }
    osc_types
}

pub(crate) fn get_sin_freq(frequency: f32, sample_clock: f32) -> f32 {
    (sample_clock * frequency * TWO_PI / SAMPLE_RATE).sin()
}

pub(crate) fn get_triangle_freq(frequency: f32, sample_clock: f32) -> f32 {
    4.0 * ((frequency / SAMPLE_RATE * sample_clock)
        - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
        .floor()).abs()
        - 1.0
}

pub(crate) fn get_square_freq(frequency: f32, sample_clock: f32) -> f32 {
    if (sample_clock * frequency / SAMPLE_RATE) % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

pub(crate) fn get_saw_freq(frequency: f32, sample_clock: f32) -> f32 {
    2.0 * ((frequency / SAMPLE_RATE * sample_clock)
        - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
        .floor()).abs()
        - 1.0
}
