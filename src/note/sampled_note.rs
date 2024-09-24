use derive_builder::Builder;
use crate::common::constants::SAMPLE_RATE;

use crate::note::constants::{DEFAULT_VOLUME, INIT_START_TIME};

pub(crate) const BUF_STORAGE_SIZE: usize = (SAMPLE_RATE as usize * 2) as usize;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug, PartialEq)]
pub(crate) struct SampledNote {
    #[builder(default = "Vec::with_capacity(BUF_STORAGE_SIZE)", setter(skip))]
    pub (crate) sample_buf: Vec<f32>,
    
    #[builder(default = "0", setter(skip))]
    pub(crate) buf_size: usize,
    
    #[builder(default = "0", setter(skip))]
    sample_index: usize,

    #[builder(default = "DEFAULT_VOLUME")]
    pub(crate) volume: f32,

    #[builder(default = "INIT_START_TIME")]
    pub(crate) start_time_ms: f32,

    #[builder(default = "INIT_START_TIME")]
    pub(crate) end_time_ms: f32,
}

impl SampledNote {
    pub (crate) fn duration_ms(&self) -> f32 {
        self.end_time_ms - self.start_time_ms
    }
    
    pub(crate) fn set_sample(&mut self, samples: &[f32], buf_size: usize) {
        self.sample_buf = samples.try_into().unwrap();
        self.buf_size = buf_size;
        self.sample_index = 0;
    }
    
    pub(crate) fn next_sample(&mut self) -> f32 {
        let sample = self.sample_buf[self.sample_index];
        self.sample_index = (self.sample_index + 1) % self.buf_size;
        sample
    }
}

#[allow(dead_code)]
pub(crate) fn default_sample_note() -> SampledNote {
    SampledNoteBuilder::default().build().unwrap()
}