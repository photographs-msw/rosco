use derive_builder::Builder;
use crate::common::constants::SAMPLE_RATE;

use crate::note::constants::{DEFAULT_VOLUME, INIT_START_TIME};
use crate::note::note_trait::BuilderWrapper;

pub(crate) const BUF_STORAGE_SIZE: usize = (SAMPLE_RATE as usize * 2) as usize;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug, PartialEq)]
pub(crate) struct SampledNote {
    #[builder(default = "Vec::with_capacity(BUF_STORAGE_SIZE)", setter(skip))]
    pub (crate) sample_buf: Vec<f32>,
    
    #[builder(default = "0", setter(skip))]
    pub(crate) buf_size: usize,
    
    #[builder(default = "0", setter(skip))]
    pub(crate) sample_index: usize,

    #[builder(default = "DEFAULT_VOLUME")]
    pub(crate) volume: f32,

    #[builder(default = "INIT_START_TIME")]
    pub(crate) start_time_ms: f32,

    #[builder(default = "INIT_START_TIME")]
    pub(crate) end_time_ms: f32,

    #[builder(default = "(INIT_START_TIME * SAMPLE_RATE as f32) as usize", setter(skip))]
    pub(crate) start_time_sample_index: usize,

    #[builder(default = "(INIT_START_TIME * SAMPLE_RATE as f32) as usize", setter(skip))]
    pub(crate) end_time_sample_index: usize,

    #[builder(default = "false")]
    pub(crate) is_looping: bool,

    #[builder(default = "0.0", setter(skip))]
    last_sample: f32,
}

#[allow(dead_code)]
impl SampledNote {
    pub(crate) fn duration_ms(&self) -> f32 {
        self.end_time_ms - self.start_time_ms
    }

    pub(crate) fn next_sample(&mut self) -> f32 {
        if self.sample_index >= self.start_time_sample_index &&
                self.sample_index < self.end_time_sample_index {
            let sample = self.sample_buf[self.sample_index];
            self.sample_index += 1;
            self.last_sample = sample;
            sample
        } else {
            // handle looping by always assigning last_sample but only checking for overflow and
            // resetting when the necessary if range check fails, avoids second if check on each
            // iteration at the cost of one extra assignment. Return last sample here rather
            // than 0.0 to avoid click
            if self.is_looping {
                self.sample_index = self.start_time_sample_index;
                self.last_sample
            } else {
                0.0
            }
        }
    }
    
    // TODO Can now add range and "scrach" kinds of access to the buffer
    
    pub(crate) fn get_sample_at(&self, index: usize) -> f32 {
        self.sample_buf[index]
    }

    pub(crate) fn set_sample_buf(&mut self, samples: &[f32], buf_size: usize) {
        self.sample_buf = samples.try_into().unwrap();
        self.buf_size = buf_size;
        
        // Set start time and index to 0
        self.start_time_ms = 0.0;
        self.start_time_sample_index = 0;
        
        // Set end time and index based on buffer length
        self.end_time_ms = (buf_size as f32) / (SAMPLE_RATE as f32) * 1000.0;  // Convert to milliseconds
        self.end_time_sample_index = buf_size;
        
        // Reset sample index to start
        self.sample_index = self.start_time_sample_index;
    }

    pub(crate) fn append_sample(&mut self, sample: f32) {
        self.sample_buf.push(sample);
        self.buf_size += 1;
    }

    pub(crate) fn reverse(&mut self) {
        self.sample_buf.reverse();
    }

    pub(crate) fn chopped(&self, num_segments: usize) -> Vec<SampledNote> {
        let mut chopped_notes = Vec::with_capacity(num_segments);
        let segment_size = self.buf_size / num_segments;
        for i in 0..num_segments {
            let start = i * segment_size;
            let end = (i + 1) * segment_size;
            let mut chopped_note = self.clone();
            chopped_note.sample_buf = self.sample_buf[start..end].to_vec();
            chopped_note.buf_size = segment_size;
            chopped_notes.push(chopped_note);
        }
        chopped_notes
    }

    // TODO Support other algorithms besides linear interpolation, which is implemented here
    pub(crate) fn stretched(&self, stretch_factor: u8) -> SampledNote {
        let mut stretched_note: SampledNote = self.clone();
        let stretched_buf_size = self.buf_size * stretch_factor as usize;
        stretched_note.sample_buf = Vec::with_capacity(stretched_buf_size);
        stretched_note.buf_size = stretched_buf_size;
        for i in 0..self.buf_size - 1 {
            let start = self.sample_buf[i];
            let end = self.sample_buf[i + 1];
            let step = (end - start) / stretch_factor as f32;
            for j in 0..stretch_factor {
                stretched_note.sample_buf.push(start + j as f32 * step);
            }
        }

        stretched_note
    }

    pub(crate) fn set_start_time_ms(&mut self, start_time_ms: f32) {
        self.start_time_ms = start_time_ms;
        self.start_time_sample_index = (start_time_ms * SAMPLE_RATE as f32) as usize;
        self.sample_index = self.start_time_sample_index;
    }

    pub(crate) fn set_end_time_ms(&mut self, end_time_ms: f32) {
        self.end_time_ms = end_time_ms;
        self.end_time_sample_index = (end_time_ms * SAMPLE_RATE as f32) as usize;
    }
}

impl BuilderWrapper<SampledNote> for SampledNoteBuilder {
    fn new() -> SampledNote {
        SampledNoteBuilder::default().build().unwrap()
    }
}

#[allow(dead_code)]
pub(crate) fn default_sample_note() -> SampledNote {
    SampledNoteBuilder::default().build().unwrap()
}
