use std::collections::VecDeque;

use derive_builder::Builder;

use crate::common::constants::SAMPLES_PER_MS;

static DEFAULT_DELAY_MIX: f32 = 1.0;
static DEFAULT_DELAY_DECAY: f32 = 0.5;
static DEFAULT_INTERVAL_DURATION_MS: f32 = 100.0;
static DEFAULT_DELAY_DURATION_MS: f32 = 20.0;
static DEFAULT_NUM_REPEATS: usize = 4;



// delay_buf: [************************************************************************* ...]
//             | duration_ms | interval_ms | duration_ms | interval_ms | duration_ms | ...
// there are num_repeats number of duration_ms sections
// duration_ms sections are width in samples of the delay window, i.e. length of each delay event
// interval_ms sections are width in samples of the silence between delay events
// as each sample comes in, insert_index updates the delay buffer rolling forward modulo
// as each sample comes in, the current delay_index is checked to see if it is in a delay window

#[allow(dead_code)]
#[derive(Builder, Clone, Debug, PartialEq)]
#[builder(build_fn(skip))]
pub(crate) struct Delay {
    
    // master level at which sample events are mixed into final output
    pub(crate) mix: f32,

    // factor for how much each sample event decays in magnitude from the previous one
    pub(crate) decay: f32,

    // duration of the silence between sample events
    pub(crate) interval_ms: f32,

    // duration of each sample event
    pub(crate) duration_ms: f32,

    // the number of sample events
    pub(crate) num_repeats: usize,

    // duration in number of samples of the silence between sample events 
    #[builder(field(private))]
    interval_num_samples: usize,

    // duration in number of samples of each sample event
    #[builder(field(private))]
    duration_num_samples: usize,

    // the size of the delay sample buffer
    #[builder(field(private))]
    delay_buf_size: usize,

    // the delay buffer 
    #[builder(field(private))]
    delay_buf: VecDeque<f32>,

    // the current index for inserting the current sample into the buffer
    #[builder(field(private))]
    insert_index: usize,

    // the current index for reading the next delay sample from the buffer
    #[builder(field(private))]
    delay_buf_index: usize,

    // is the delay active or has it reached its last event
    #[builder(field(private))]
    is_active: bool,
}

#[allow(dead_code)]
impl DelayBuilder {
    
    pub(crate) fn build(&mut self) -> Result<Delay, String> {
        let mix = self.mix.unwrap_or(DEFAULT_DELAY_MIX);
        let decay = self.decay.unwrap_or(DEFAULT_DELAY_DECAY);
        let interval_ms = self.interval_ms.unwrap_or(DEFAULT_INTERVAL_DURATION_MS);
        let duration_ms = self.duration_ms.unwrap_or(DEFAULT_DELAY_DURATION_MS);
        let num_repeats = self.num_repeats.unwrap_or(DEFAULT_NUM_REPEATS);

        let interval_num_samples =
            interval_ms as usize * SAMPLES_PER_MS as usize;
        let duration_num_samples =
            duration_ms as usize * SAMPLES_PER_MS as usize;
        
        let mut delay_buf_size = 0;
        if num_repeats > 0 {
            delay_buf_size = (
                (num_repeats as f32 * duration_ms + ((num_repeats - 1) as f32 * interval_ms)) *
                    SAMPLES_PER_MS) as usize;
        }

        let mut buffer = VecDeque::with_capacity(delay_buf_size);
        for _ in 0..delay_buf_size {
            buffer.push_back(0.0);
        }
        
        let delay_buf = buffer.clone();
        let insert_index = 0;
        let delay_buf_index = 0;
        let is_active = true;
        
        Ok(
            Delay {
                mix,
                decay,
                interval_ms,
                duration_ms,
                num_repeats,
                interval_num_samples,
                duration_num_samples,
                delay_buf_size,
                delay_buf,
                insert_index,
                delay_buf_index,
                is_active,
            }
        )
    }
}

#[allow(dead_code)]
impl Delay {
    
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        
        // TEMP DEBUG
        println!("sample: {}, insert_index: {}, delay_buf_index: {}, is_active: {}", sample, self.insert_index, self.delay_buf_index, self.is_active);
        
        // rolling update of the samples in the delay buffer
        self.delay_buf.insert(self.insert_index % self.delay_buf_size, sample);
        self.insert_index += 1;
        
        // Is the delay index into the buffer in a window that should be delayed?
        let delay_window_index = self.in_delay_window(self.delay_buf_index);
        if delay_window_index == self.num_repeats {
            self.is_active = false;
        }
        if self.is_active && delay_window_index > 0 {
            let delay_sample = self.delay_buf[self.delay_buf_index];
            self.delay_buf_index += 1;
            // The decay is the initial constant factor over which window we are in, i.e. gets
            // smaller for each delay step we are on. Linear decay here because we are dividing.
            let decay_factor = self.decay / delay_window_index as f32;
            // Normalize the return. One part is the current sample weighted with the
            // inverse of the decay factor applied to the sample buffer sample. The other is the
            // weighted sample buffer sample. Adding them means that as the sample decays more
            // of the result is the current sample.
            // Handle the case if initialized buffer or 0 delay sample
            if delay_sample > 0.0 {
                
                // TEMP DEBUG
                println!("sample: {}, delay_sample: {}, decay_factor: {}", sample, delay_sample, decay_factor);
                
                ((1.0 - decay_factor) * sample) + (decay_factor * self.mix * delay_sample)
            } else {
               sample 
            }
        } else {
            self.delay_buf_index += 1;
            sample
        }
    }
    
    pub(crate) fn reset(&mut self) {
        self.insert_index = 0;
        self.delay_buf_index = 0;
        self.is_active = true;
        for _ in 0..self.delay_buf_size {
            self.delay_buf.push_back(0.0);
        }
    }
    
    // returns 0 if not in delay window
    // returns positive integer corresponding to the delay window index if in delay window, this
    //  is the delay window index + 1, i.e. 1-based index
    fn in_delay_window(&mut self, index: usize) -> usize {
        for i in 0..self.num_repeats {
            let left_bound = i * (self.duration_num_samples + self.interval_num_samples);
            if index >= left_bound && index < left_bound + self.duration_num_samples {
                return i + 1;
            }
        }

        0
    }
}

#[allow(dead_code)]
pub(crate) fn default_delay() -> Delay {
    DelayBuilder::default()
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn no_op_delay() -> Delay {
    DelayBuilder::default()
        .num_repeats(0)
        .build().unwrap()
}
