use std::collections::VecDeque;

use derive_builder::Builder;

use crate::common::constants::SAMPLES_PER_MS;
use crate::common::pair::Pair;

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

    // the number of sample events
    pub(crate) auto_reset: bool,

    // duration in number of samples of the silence between sample events 
    #[builder(field(private))]
    interval_num_samples: usize,

    // duration in number of samples of each sample event
    #[builder(field(private))]
    duration_num_samples: usize,

    // the size of the delay sample buffer
    #[builder(field(private))]
    window_size: usize,

    // the delay buffer 
    #[builder(field(private))]
    sample_buffer: VecDeque<f32>,

    // the current index for inserting the current sample into the buffer
    #[builder(field(private))]
    insert_index: usize,

    // the current index for reading the next delay sample from the buffer
    #[builder(field(private))]
    delay_sample_index: usize,

    // is the delay active or has it reached its last event
    #[builder(field(private))]
    is_active: bool,
    
    // boundaries of sample indexes in delay windows or in intervals between delay windows
    delay_windows: Vec<Pair<usize>>,
}

fn build_delay_windows(duration_num_samples: usize, interval_num_samples: usize,
                       num_repeats: usize) -> Vec<Pair<usize>> {

    let mut delay_windows = Vec::new();
    let samples_total = duration_num_samples + interval_num_samples;
    for i in 0..num_repeats {
        delay_windows.push(
            Pair (i * samples_total,
                  (i * samples_total) + duration_num_samples))
    }

    delay_windows
}

fn in_delay_window(index: usize, delay_windows: &Vec<Pair<usize>>) -> usize {
    for (i, window) in delay_windows.iter().enumerate() {
        if window.0 <= index && index < window.1 {
            return i;
        }
    }

    0 
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

        let interval_num_samples =
            interval_ms as usize * SAMPLES_PER_MS as usize;
        let duration_num_samples =
            duration_ms as usize * SAMPLES_PER_MS as usize;
        
        let mut window_size = 0;
        if num_repeats > 0 {
            window_size = (
                (num_repeats as f32 * duration_ms + ((num_repeats - 1) as f32 * interval_ms)) *
                    SAMPLES_PER_MS) as usize;
        }

        // Not initialized because expect user to initialize with init_delay_buf()
        let sample_buffer = VecDeque::with_capacity(window_size);
        
        let insert_index = 0;
        let delay_sample_index = 0;
        let is_active = true;
        
        Ok(
            Delay {
                // public
                mix,
                decay,
                interval_ms,
                duration_ms,
                num_repeats,
                auto_reset,
                // private 
                interval_num_samples,
                duration_num_samples,
                window_size,
                sample_buffer,
                insert_index,
                delay_sample_index,
                is_active,
                delay_windows: build_delay_windows(duration_num_samples, interval_num_samples,
                                                   num_repeats) 
            }
        )
    }
}

#[allow(dead_code)]
impl Delay {
    
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        
        // rolling update of the samples in the delay buffer
        self.sample_buffer.insert(self.insert_index % self.window_size, sample);
        self.insert_index += 1;
        
        // let delay_window_index = self.in_delay_window(self.delay_buf_index);

        // TEMP DEBUG
        // println!("BEFORE CHECK is_active {} delay_window_index {}", self.is_active, delay_window_index);


        // TEMP DEBUG
        // println!("AFTER CHECK is_active {} delay_window_index {}", self.is_active, delay_window_index);

        // Is the delay index into the buffer in a window that should be delayed?
        let delay_window_index = in_delay_window(self.insert_index, &self.delay_windows);
        if delay_window_index == self.num_repeats {
            if !self.auto_reset {
                self.is_active = false;
            } else {
                self.reset();
            }
        }
        
        if self.is_active && delay_window_index > 0 {

            // TEMP DEBUG
            // println!("IN LOOP is_active {} delay_window_index {}", self.is_active, delay_window_index);

            let delay_sample = self.sample_buffer[self.delay_sample_index];
            self.delay_sample_index += 1;
            // The decay is the initial constant factor over which window we are in, i.e. gets
            // smaller for each delay step we are on. Linear decay here because we are dividing.
            let decay_factor = self.decay / delay_window_index as f32;
            // Normalize the return. One part is the current sample weighted with the
            // inverse of the decay factor applied to the sample buffer sample. The other is the
            // weighted sample buffer sample. Adding them means that as the sample decays more
            // of the result is the current sample.
            // Handle the case if initialized buffer or 0 delay sample
            ((1.0 - decay_factor) * (1.0 - self.mix) * sample) +
                (decay_factor * self.mix * delay_sample)
            
            // TEMP DEBUG
            // sample
        } else {
            sample
        }
    }
    
    pub(crate) fn reset(&mut self) {
        self.insert_index = 0;
        self.delay_sample_index = 0;
        self.is_active = true;
        for _ in 0..self.window_size {
            self.sample_buffer.push_back(0.0);
        }
    }

    pub(crate) fn init_delay_buf(&mut self, samples: Vec<f32>) {
        for i in 0..self.window_size {
            self.sample_buffer.push_back(samples[i]);
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
