use derive_builder::Builder;

static SAMPLE_BUFFER_SIZE: usize = 20;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug, PartialEq)]
pub(crate) struct Flanger {
    #[builder(default = "SAMPLE_BUFFER_SIZE")]
    pub(crate) window_size: usize,
    
    #[builder(setter(custom))]
    sample_buffer: Vec<f32>,

    #[builder(default = "1.0")]
    mix: f32, 
    
    #[builder(default = "0", setter(skip))]
    insert_index: usize,
}

#[allow(dead_code)]
impl FlangerBuilder {
    // Initialize sample buffer to size with zeros. THis makes the first n calls up to window size
    // no-ops but then after that the buffer is full and the effect is applied. Allows us to 
    // avoid having to check if the buffer is full in the apply_effect method, at the cst of
    // a slight delay in the effect being applied.
    pub(crate) fn sample_buffer(&mut self) -> &mut Self {
        let window_size = self.window_size.unwrap();
        let mut buffer: Vec<f32> = Vec::with_capacity(window_size);
        // init to silence
        buffer.resize(window_size, 0.0);
        for i in 0..window_size {
            buffer[i] = (0.0);
        }
        self.sample_buffer = Some(buffer);
        self 
    }
}

#[allow(dead_code)]
impl Flanger {
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        // circular buffer of most recent samples in window, effect uses the oldest sample
        self.sample_buffer[self.insert_index % self.window_size] = sample;
        self.insert_index += 1;
        
        sample + (self.mix * self.sample_buffer[self.window_size - 1])
    }
}

#[allow(dead_code)]
pub(crate) fn default_flanger() -> Flanger {
    FlangerBuilder::default()
        .window_size(SAMPLE_BUFFER_SIZE)
        .sample_buffer()
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn no_op_flanger() -> Flanger {
    FlangerBuilder::default()
        .window_size(0)
        .sample_buffer()
        .build().unwrap()
}
