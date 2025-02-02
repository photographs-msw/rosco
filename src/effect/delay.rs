use std::collections::VecDeque;

use derive_builder::Builder;

use crate::common::constants::SAMPLES_PER_MS;

static DEFAULT_DELAY_MIX: f32 = 1.0;
static DEFAULT_DELAY_DECAY: f32 = 0.5;
static DEFAULT_INTERVAL_DURATION_MS: f32 = 100.0;
static DEFAULT_DELAY_DURATION_MS: f32 = 20.0;
static DEFAULT_NUM_REPEATS: usize = 4;
static SAMPLE_BUFFER_INIT_SIZE: usize = 10;
static MAX_NUM_SAMPLE_DELAY_WINDOWS: usize = 16;


// delay_buf: [************************************************************************* ...]
//             | duration_ms | interval_ms | duration_ms | interval_ms | duration_ms | ...
// there are num_repeats number of duration_ms sections
// duration_ms sections are width in samples of the delay window, i.e. length of each delay event
// interval_ms sections are width in samples of the silence between delay events
// as each sample comes in, insert_index updates the delay buffer rolling forward modulo
// as each sample comes in, the current delay_index is checked to see if it is in a delay window
// once the index gets to the end of the delay window, num_repeats increments. If the window
//  has repeated num_repeats times, it's put back in the pool. If it has not, a new window is
//  is pulled from the pool and it starts recording samples


#[allow(dead_code)]
#[derive(Builder, Clone, Debug, PartialEq)]
#[builder]
pub(crate) struct SampleManager {
    // the size of the delay sample buffer
    #[builder(field(private))]
    sample_buffer_size: usize,

    #[builder(field(private))]
    sample_buffer: VecDeque<f32>,

    // boundaries of sample indexes in delay windows or in intervals between delay windows
    // true if in delay window, false if in interval
    delay_windows: Vec<bool>,
    
    // the current index for reading the next delay sample from the buffer
    #[builder(default = "0")]
    sample_buffer_read_index: usize,

    // the current index for writing the next delay sample from the buffer
    #[builder(default = "0")]
    sample_buffer_write_index: usize,

    // leader buffer, we don't start reading and incrementing other buffers until we have written
    // this many initializing samples
    #[builder(default = "0")]
    init_buffer_index: usize,
    
    // which delay window we are in, used to calculate decay factor
    #[builder(default = "0")]
    delay_window_index: usize,
    
    // position in bit vector of entire length of all delay windows
    #[builder(default = "0")]
    delay_windows_index: usize,

    // true if the sample manager can still write more samples
    #[builder(default = "false")]
    is_full: bool,

    #[builder(default = "true")]
    is_active: bool,
}

// Not initialized because expect user to initialize with init_delay_buf()
// let sample_buffer = VecDeque::with_capacity(window_size);

#[allow(dead_code)]
impl SampleManager {
    
    pub(crate) fn next_sample(&mut self, sample: f32) -> f32 {
        let mut delay_sample = 0f32;

        if !self.is_active {
            return 0f32;
        }
        
        if self.init_buffer_index < SAMPLE_BUFFER_INIT_SIZE {
            self.sample_buffer.push_back(sample);
            self.init_buffer_index += 1;
            return 0f32;
        }

        if !self.is_full {
            self.sample_buffer.push_back(sample);
            self.sample_buffer_write_index += 1;
        }
        if self.sample_buffer_write_index == self.sample_buffer_size {
            self.is_full = true;
        }
        
        if self.delay_windows[self.delay_windows_index] {
            delay_sample =
                self.sample_buffer[self.sample_buffer_read_index % self.sample_buffer_size];
            // If this is the first sample in the delay window, increment the delay window index
            if self.sample_buffer_read_index == 0 {
                self.delay_window_index += 1;
            }
            self.sample_buffer_read_index += 1;
        }
        self.delay_windows_index += 1;
        if self.delay_windows_index == self.delay_windows.len() {
            self.is_active = false;
        }

        delay_sample 
    }
    
    pub(crate) fn reset(&mut self) {
        self.sample_buffer_read_index = 0;
        self.sample_buffer_write_index = 0;
        self.init_buffer_index = 0;
        self.delay_windows_index = 0;
        self.is_full = false;
        self.is_active = true;
        self.delay_window_index = 0;
        for i in 0..self.sample_buffer_size {
            self.sample_buffer[i] = 0f32;
        }
    }
}

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

    // if true, the delay will reset to the beginning when the last sample event has played
    pub(crate) auto_reset: bool,

    // the number of simultaneous delay windows that can be active 
    pub(crate) concurrency_factor: usize,
    
    // complement of mix, private compute at build time because it's constant
    #[builder(field(private))]
    mix_complement: f32,
    
    // the size of the delay sample buffer
    #[builder(field(private))]
    window_size: usize,

    // boundaries of sample indexes in delay windows or in intervals between delay windows
    #[builder(field(private))]
    delay_windows: Vec<bool>,
    
    // a pool of sample managers, each of which is managing a sample buffer and getting the
    // next sample from the buffer
    #[builder(field(private))]
    sample_managers: Vec<SampleManager>,
    
}

fn build_delay_windows(duration_num_samples: usize, interval_num_samples: usize,
                       num_repeats: usize) -> Vec<bool> {

    let mut delay_windows = Vec::new();
    let samples_total = (duration_num_samples * num_repeats) +
        (interval_num_samples * num_repeats - 1);
    
    let mut in_window = true;
    let mut in_window_index: usize = 0;
    for i in 0..samples_total {
        if in_window {
            delay_windows.push(true);
        } else {
            delay_windows.push(false);
        }
        
        in_window_index += 1;
        if in_window && in_window_index == duration_num_samples {
            in_window = false;
            in_window_index = 0;
        } else if !in_window && in_window_index == interval_num_samples {
            in_window = true;
            in_window_index = 0;
        }
    }

    delay_windows
}

#[allow(dead_code)]
impl DelayBuilder {
    
    pub(crate) fn build(&mut self) -> Result<Delay, String> {
        let mix = self.mix.unwrap_or(DEFAULT_DELAY_MIX);
        let decay = self.decay.unwrap_or(DEFAULT_DELAY_DECAY);
        let interval_ms = self.interval_ms.unwrap_or(DEFAULT_INTERVAL_DURATION_MS);
        let duration_ms = self.duration_ms.unwrap_or(DEFAULT_DELAY_DURATION_MS);
        let num_repeats = self.num_repeats.unwrap_or(DEFAULT_NUM_REPEATS);
        let auto_reset = self.auto_reset.unwrap_or(false);
        
        let concurrency_factor =
            self.concurrency_factor.unwrap_or(MAX_NUM_SAMPLE_DELAY_WINDOWS);
        
        let duration_num_samples = duration_ms as usize * SAMPLES_PER_MS as usize;
        let interval_num_samples = interval_ms as usize * SAMPLES_PER_MS as usize;
        let mut sample_managers: Vec<SampleManager> = Vec::with_capacity(concurrency_factor);
        for i in 0..concurrency_factor {
            sample_managers.push(
                SampleManagerBuilder::default()
                    .sample_buffer_size(duration_num_samples)
                    .sample_buffer(VecDeque::with_capacity(duration_num_samples))
                    .delay_windows(build_delay_windows(
                        duration_num_samples,
                        interval_num_samples,
                        num_repeats))
                    .build().unwrap()
            );
        }
        
        let mix_complement = 1.0 - mix;
        let window_size =
            duration_ms as usize * SAMPLES_PER_MS as usize;
        
        
        Ok(
            Delay {
                // public
                mix,
                decay,
                interval_ms,
                duration_ms,
                num_repeats,
                auto_reset,
                concurrency_factor,
                // private
                mix_complement,
                window_size,
                delay_windows: build_delay_windows(duration_num_samples, interval_num_samples,
                                                   num_repeats),
                sample_managers,
            }
        )
    }
}

#[allow(dead_code)]
impl Delay {
    
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        
        let mut delay_sample = 0f32;
        
        // go forward through the sample managers and get the next delay sample from each
        // any that aren't active will return 0
        // count number of delay samples returned so we can divide total, use mean to normalize
        let mut num_delay_samples = 0;
        for i in 0..self.concurrency_factor {
            let sample_manager = &mut self.sample_managers[i];
            
            let next_delay_sample = sample_manager.next_sample(sample);
            if next_delay_sample != 0f32 {
                // add each sample returned factored by the decay for that sample manager, each
                // might be in a different delay window
                delay_sample +=
                    next_delay_sample * self.decay.powi(sample_manager.delay_window_index as i32);
                num_delay_samples += 1;
            }
            
            // spawn the next active sample manager if this one is full, unless all are full
            if sample_manager.is_active && sample_manager.is_full {
                for j in 0..self.concurrency_factor {
                    if !self.sample_managers[j].is_active {
                        self.sample_managers[j].reset();
                        break;
                    }
                }
            } else if !sample_manager.is_active {
                // just became inactive on this iteration, so if we are auto resetting, reset it
                if self.auto_reset {
                    sample_manager.reset();
                }
            }
        }
        // normalize the sum of the delay samples by the number of delay samples
        delay_sample /= num_delay_samples as f32;
        
        // return the sample added to the delay, each factored by mix factor
        self.mix_complement * sample + self.mix * delay_sample
    }
    
    pub(crate) fn reset(&mut self) {
        for i in 0.. self.concurrency_factor {
            self.sample_managers[i].reset();
        }
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
