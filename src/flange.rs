use std::collections::VecDeque;
use derive_builder::Builder;

static SAMPLE_BUFFER_SIZE: usize = 20;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug, PartialEq)]
pub(crate) struct Flange {
    #[builder(default = "SAMPLE_BUFFER_SIZE")]
    window_size: usize,
    
    #[builder(default = "VecDeque::with_capacity(SAMPLE_BUFFER_SIZE)", setter(custom))]
    sample_buffer: VecDeque<f32>,
    
    #[builder(default = "0", setter(skip))]
    insert_index: usize,
}

#[allow(dead_code)]
impl FlangeBuilder {
    // Initialize sample buffer to size with zeros. THis makes the first n calls up to window size
    // no-ops but then after that the buffer is full and the effect is applied. Allows us to 
    // avoid having to check if the buffer is full in the apply_effect method, at the cst of
    // a slight delay in the effect being applied.
    fn sample_buffer(&mut self) -> &mut Self {
        let mut buffer: VecDeque<f32> = VecDeque::with_capacity(self.window_size.unwrap());
        for _ in 0..self.window_size.unwrap() {
            buffer.push_back(0.0);
        }
        self.sample_buffer = Some(buffer.clone());
        self 
    }
}

#[allow(dead_code)]
impl Flange {
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        // Fill the buffer in a rolling window with the current sample
        // TODO this is a bit of a hack, we should probably just use a circular buffer
        self.sample_buffer.insert(self.insert_index % self.window_size, sample);
        self.insert_index += 1;
        
        sample + self.sample_buffer[self.window_size - 1]
    }
}

#[allow(dead_code)]
pub(crate) fn default_flange() -> Flange {
    FlangeBuilder::default()
        .window_size(SAMPLE_BUFFER_SIZE)
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn no_op_flange() -> Flange {
    FlangeBuilder::default()
        .window_size(0)
        .build().unwrap()
}
