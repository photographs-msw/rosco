use std::collections::{LinkedList, VecDeque};

use derive_builder::Builder;

use crate::common::constants::SAMPLES_PER_MS;
use crate::common::float_utils::float_eq;
use crate::common::float_utils::float_geq;
use crate::common::float_utils::float_leq;
use crate::common::float_utils::float_neq;

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
   
    num_delay_windows: i8,
    
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
    #[builder(default = "1")]
    cur_delay_window: usize,
    
    // position in bit vector of entire length of all delay windows
    #[builder(default = "0")]
    delay_windows_index: usize,
    
    // false if the sample manager can still write more samples
    #[builder(default = "false")]
    is_full: bool,

    // true if the sample manager hasn't finished going through its delay windows
    #[builder(default = "true")]
    is_active: bool,
    
    // true if the sample manager is adding initial samples before starting to process its
    // delay windows by reading them back and advancing
    #[builder(default = "true")]
    is_initializing: bool,

    #[builder(default = "true")]
    is_in_delay_window: bool,

    #[builder(default = "false")]
    is_in_interval: bool,
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
        
        if self.is_initializing {
            self.sample_buffer.push_back(sample);
            self.init_buffer_index += 1;
            if self.init_buffer_index == SAMPLE_BUFFER_INIT_SIZE {
                self.is_initializing = false;
            }
            return 0f32;
        }

        if !self.is_full {
            self.sample_buffer.push_back(sample);
            self.sample_buffer_write_index += 1;
        }
        if self.sample_buffer_write_index == self.sample_buffer_size {
            self.is_full = true;
        }
        
        // check if we are in a delay window or an interval by checking current delay window value
        if self.delay_windows[self.delay_windows_index] {
            delay_sample =
                *self.sample_buffer
                    .get(self.sample_buffer_read_index % self.sample_buffer_size).unwrap();
            // If this is the first sample in the delay window, increment the delay window index
            if self.sample_buffer_read_index == 0 {
                self.cur_delay_window += 1;
            }
            self.sample_buffer_read_index += 1;
        }
        self.delay_windows_index += 1;
        if self.delay_windows_index == self.sample_buffer_size {
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
        self.cur_delay_window = 0;
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
    
    // a pool of sample managers, each of which can manage a sample buffer, allocated initially
    // and then used as a stack to provide active SampleManagers as needed and return inactive
    // ones to the pool
    #[builder(field(private))]
    sample_managers_pool: Vec<SampleManager>,

    #[builder(field(private))]
    active_sample_managers: Vec<bool>, 
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
        
        let concurrency_factor =
            self.concurrency_factor.unwrap_or(MAX_NUM_SAMPLE_DELAY_WINDOWS);
        
        let duration_num_samples = duration_ms as usize * SAMPLES_PER_MS as usize;
        let interval_num_samples = interval_ms as usize * SAMPLES_PER_MS as usize;
        // create the pool of SampleManagers
        let mut sample_managers_pool: Vec<SampleManager> = Vec::with_capacity(concurrency_factor);
        for i in 0..concurrency_factor {
            sample_managers_pool.push(
                SampleManagerBuilder::default()
                    .sample_buffer_size(duration_num_samples)
                    .sample_buffer(VecDeque::with_capacity(duration_num_samples))
                    .delay_windows(build_delay_windows(
                        duration_num_samples,
                        interval_num_samples,
                        num_repeats))
                    .num_delay_windows(num_repeats as i8)
                    .build().unwrap()
            );
        }
        // initialize the delay with one active SampleManager
        let mut active_sample_managers = Vec::with_capacity(concurrency_factor);
        active_sample_managers.push(false);
        active_sample_managers[0] = true;
        
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
                concurrency_factor,
                // private
                mix_complement,
                window_size,
                delay_windows: build_delay_windows(duration_num_samples, interval_num_samples,
                                                   num_repeats),
                sample_managers_pool,
                active_sample_managers,
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
        let mut indexes_to_release_to_pool: Vec<usize> = Vec::new();
        let mut num_to_take_from_pool = 0;
        for (i, is_active) in self.active_sample_managers.iter().enumerate() {
            if !is_active {
                continue;
            }

            let sample_manager = self.sample_managers_pool.get_mut(i).unwrap();
            let next_delay_sample = sample_manager.next_sample(sample);
            // add each sample returned factored by the decay for that sample manager, each
            // might be in a different delay window
            delay_sample +=
                next_delay_sample * self.decay.powi(sample_manager.cur_delay_window as i32);
            num_delay_samples += 1;

            // spawn the next active sample manager if this one is still active and is full,
            // unless the pool is exhausted 
            if sample_manager.is_active && sample_manager.is_full {
                num_to_take_from_pool += 1;
            } else if !sample_manager.is_active {
                // just became inactive on this iteration, record index to release to pool
                indexes_to_release_to_pool.push(i);
                sample_manager.reset();
            }
        }
        // do bookkeeping to release sample_managers to pool
        for idx in indexes_to_release_to_pool.iter() {
            self.active_sample_managers[*idx] = false;
        }
        // do bookkeeping to take available sample_managers from pool
        let mut taken_count = 0;
        for manager_is_active in self.active_sample_managers.iter_mut() {
            let is_active = *manager_is_active;
            if !is_active {
                *manager_is_active = true;
                taken_count += 1;
                if taken_count == num_to_take_from_pool {
                    break;
                }
            }
        }

        // normalize the sum of the delay samples by the number of delay samples
        delay_sample /= num_delay_samples as f32;
        // if we don't match signs then the delay sample has the effect of cancelling out the sample
        if float_leq(sample, 0.0) && float_geq(delay_sample, 0.0) {
            delay_sample *= -1.0;
        }
        sample + (self.mix * delay_sample)
    }

    pub(crate) fn reset(&mut self) {
        for i in 0.. self.concurrency_factor {
            self.sample_managers_pool[i].reset();
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
