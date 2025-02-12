use std::collections::VecDeque;
use std::sync::Arc;

use derive_builder::Builder;

use crate::common::constants::SAMPLES_PER_MS;

pub(crate) const PREDELAY_BUFFER_SIZE: usize = 20;

static DEFAULT_DELAY_MIX: f32 = 1.0;
static DEFAULT_DELAY_DECAY: f32 = 0.5;
static DEFAULT_INTERVAL_DURATION_MS: f32 = 100.0;
static DEFAULT_DELAY_DURATION_MS: f32 = 20.0;
static DEFAULT_NUM_REPEATS: usize = 4;
static MAX_NUM_SAMPLE_DELAY_WINDOWS: usize = 128;


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
    sample_buffer: Arc<VecDeque<f32>>,

    // boundaries of sample indexes in delay windows or in intervals between delay windows
    // true if in delay window, false if in interval
    delay_windows: Vec<bool>,
   
    num_delay_windows: usize,
    
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

    #[builder(default = "true")]
    is_pre_delay: bool,
        
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
        
        // if we are in the pre-delay buffer, increment the write index, add the sample to the
        // buffer and return 0
        if self.is_pre_delay {
            let buffer = Arc::make_mut(&mut self.sample_buffer);
            buffer.push_back(sample);
            self.sample_buffer_write_index += 1;
            if self.sample_buffer_write_index == PREDELAY_BUFFER_SIZE {
                self.is_pre_delay = false;
            }
            return 0f32;
        }
        
        // if the buffer holding the samples being repeated in each delay window is not full,
        // add the sample to the buffer
        if !self.is_full {
            let buffer = Arc::make_mut(&mut self.sample_buffer);
            buffer.push_back(sample);
            self.sample_buffer_write_index += 1;
        }
        if self.sample_buffer_write_index == self.sample_buffer_size {
            self.is_full = true;
        }
        
        // check if we are in a delay window or an interval by checking current delay window value
        if self.delay_windows[self.delay_windows_index] {
            let read_index = self.sample_buffer_read_index % self.sample_buffer_size;
            delay_sample = *self.sample_buffer.get(read_index).unwrap_or(&0.0);
            // If this is the first sample in the delay window, increment the delay window index
            if read_index == 0 {
                self.cur_delay_window += 1;
            }
            self.sample_buffer_read_index += 1;
        }
        
        // check for reaching the end of the delay windows
        self.delay_windows_index += 1;
        if self.delay_windows_index == self.delay_windows.len() {
            self.reset();
        }

        delay_sample 
    }
    
    pub(crate) fn reset(&mut self) {
        self.sample_buffer_read_index = 0;
        self.sample_buffer_write_index = 0;
        self.init_buffer_index = 0;
        self.cur_delay_window = 0;
        self.delay_windows_index = 0;
        self.is_full = false;
        self.is_active = true;
        self.is_pre_delay = true;
        self.is_in_delay_window = true;
        self.is_in_interval = false;

        let buffer = Arc::make_mut(&mut self.sample_buffer);
        buffer.clear();
    }

    pub(crate) fn dump_print(&self) {
        if self.is_active {
            println!("--------------------------------");
            println!("sample_buffer_size: {}", self.sample_buffer_size);
            println!("sample_buffer_read_index: {}", self.sample_buffer_read_index);
            println!("sample_buffer_write_index: {}", self.sample_buffer_write_index);
            println!("init_buffer_index: {}", self.init_buffer_index);
            println!("cur_delay_window: {}", self.cur_delay_window);
            println!("delay_windows_index: {}", self.delay_windows_index);
            println!("is_full: {}", self.is_full);
            println!("is_active: {}", self.is_active);
            println!("is_in_delay_window: {}", self.is_in_delay_window);
            println!("is_in_interval: {}", self.is_in_interval);
            println!("--------------------------------");
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

    // the number of simultaneous delay windows that can be active 
    pub(crate) concurrency_factor: usize,

    pub(crate) num_active_sample_managers: usize,
    
    // complement of mix, private compute at build time because it's constant
    #[builder(field(private))]
    mix_complement: f32,
    
    // the size of the delay sample buffer
    // #[builder(field(private))]
    // window_size: usize,

    // boundaries of sample indexes in delay windows or in intervals between delay windows
    #[builder(field(private))]
    delay_windows: Vec<bool>,
    
    // a pool of sample managers, each of which can manage a sample buffer, allocated initially
    // and then used as a stack to provide active SampleManagers as needed and return inactive
    // ones to the pool
    // #[builder(field(private))]
    // sample_managers_pool: Vec<SampleManager>,

    #[builder(field(private))]
    active_sample_managers: Vec<SampleManager>
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
        // let duration_num_samples = 70 * SAMPLES_PER_MS as usize;
        let interval_num_samples = interval_ms as usize * SAMPLES_PER_MS as usize;
        // let interval_num_samples = 50 as usize * SAMPLES_PER_MS as usize;
        // create the pool of SampleManagers
        
        // TODO GET RID OF POOL
        // let mut sample_managers_pool: Vec<SampleManager> = Vec::with_capacity(concurrency_factor);
        // for _ in 0..concurrency_factor {
        //     sample_managers_pool.push(
        //         SampleManagerBuilder::default()
        //             .sample_buffer_size(duration_num_samples)
        //             .sample_buffer(Arc::new(VecDeque::with_capacity(duration_num_samples)))
        //             .delay_windows(build_delay_windows(
        //                 duration_num_samples,
        //                 interval_num_samples,
        //                 num_repeats))
        //             .num_delay_windows(num_repeats)
        //             .build().unwrap()
        //     );
        // }
        // initialize the delay with one active SampleManager
        let mut active_sample_managers = Vec::new();
        active_sample_managers.push(
            SampleManagerBuilder::default()
                .sample_buffer_size(duration_num_samples)
                .sample_buffer(Arc::new(VecDeque::with_capacity(duration_num_samples)))
                .delay_windows(build_delay_windows(
                    duration_num_samples,
                    interval_num_samples,
                    num_repeats))
                .num_delay_windows(num_repeats)
                .build().unwrap()
        );
        
        let mix_complement = 1.0 - mix;
        // let window_size =
        //     duration_ms as usize * SAMPLES_PER_MS as usize;
        
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
                // window_size,
                delay_windows: build_delay_windows(duration_num_samples, interval_num_samples,
                                                   num_repeats),
                active_sample_managers,
                num_active_sample_managers: 1,
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
        let mut push_count = 0;
        for (i, sample_manager)
                in self.active_sample_managers.iter_mut().enumerate() {
            // capture whether this sample fetch made the manager become full, in which case
            // we need to add another manager
            let manager_is_full_before_fetch = sample_manager.is_full;

            // TEMP DEBUG
            // println!("sample manager {} is active", i);
            // sample_manager.dump_print();

            let next_delay_sample = sample_manager.next_sample(sample);

            // TEMP DEBUG
            // println!("next_delay_sample: {} {}", i, next_delay_sample);
            // sample_manager.dump_print();

            // add each sample returned factored by the decay for that sample manager, each
            // might be in a different delay window
            delay_sample +=
                next_delay_sample * self.decay.powi(sample_manager.cur_delay_window as i32);
            num_delay_samples += 1;

            // spawn the next active sample manager if this one is still active and is full,
            // unless the pool is exhausted 
            // TODO BUG HERE
            // actual chevk should be are all managers full, only then need a new one
            // MOVE UP TO MANAGER
            if !manager_is_full_before_fetch && sample_manager.is_full {
                push_count += 1;
            }

            // TODO THIS SEEMS LIKE A BUG TOO
            // MOVE UP TO MANAGER
            // should just reset and stay active because we will never need fewer
            // concurrent buffers then the max number we spawned each time all 
            // the active managers are full. By definition can't need less
            // because buffers sizes are fixed, sample internvals are fixed
            // effect never turns off
            // if !sample_manager.is_active {
            //     sample_manager.reset();
                // just became inactive on this iteration, record index to release to pool
                // indexes_to_release_to_pool.push(i);
            // }
        }

        // if push_count > 0 {
        //     let duration_num_samples = self.duration_ms as usize * SAMPLES_PER_MS as usize;
        //     let interval_num_samples = self.interval_ms as usize * SAMPLES_PER_MS as usize;

        //     self.active_sample_managers.push(
        //         SampleManagerBuilder::default()
        //             .sample_buffer_size(duration_num_samples)
        //             .sample_buffer(Arc::new(VecDeque::with_capacity(duration_num_samples)))
        //             .delay_windows(build_delay_windows(
        //                 duration_num_samples,
        //                 interval_num_samples,
        //                 self.num_repeats))
        //             .num_delay_windows(self.num_repeats)
        //         .build().unwrap()
        //     );
        // }

        // do bookkeeping to release sample_managers to pool
        // for idx in indexes_to_release_to_pool.iter() {
        //     self.active_sample_managers[*idx] = false;
        //     self.num_active_sample_managers -= 1;
        // }
        // do bookkeeping to take available sample_managers from pool
        // let mut taken_count = 0;
        // if num_to_take_from_pool > 0 {
        //     for (i, manager_is_active)
        //             in self.active_sample_managers.iter_mut().enumerate() {
        //         let is_active = *manager_is_active;
        //         if !is_active {
        //             *manager_is_active = true;
        //             self.sample_managers_pool[i].reset();
        //             self.num_active_sample_managers += 1;
        //             taken_count += 1;
        //             if taken_count == num_to_take_from_pool {
        //                 break;
        //             }
        //         }
        //     }
        // }
        
        // if (float_leq(delay_sample, 0.0) && float_geq(sample, 0.0)) || 
        //     (float_geq(delay_sample, 0.0) && float_leq(sample, 0.0)) {
        //     delay_sample *= -1.0;
        // }

        // TEMP DEBUG
        // println!("sample {}, delay_sample: {}", sample, delay_sample);

        // normalize the sum of the delay samples by the number of delay samples
        delay_sample /= num_delay_samples as f32;

        // TEMP DEBUG
        // println!("sample {}, delay_sample: {}", sample, delay_sample);

        self.mix_complement * sample + (self.mix * delay_sample)
    }

    // pub(crate) fn reset(&mut self) {
    //     for i in 0.. self.concurrency_factor {
    //         self.sample_managers_pool[i].reset();
    //     }
    // }
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
