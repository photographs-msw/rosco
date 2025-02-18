use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use derive_builder::Builder;

static SAMPLE_BUFFER_SIZE: usize = 20;
static DEFAULT_WINDOW_SIZE: usize = 12;
static DEFAULT_MIX: f32 = 0.5;

#[derive(Builder, Debug)]
pub(crate) struct Flanger {
    // The size of the sample window
    #[builder(default = "SAMPLE_BUFFER_SIZE")]
    window_size: usize,

    // The buffer holding samples for the flanger effect
    #[builder(field(private),
      default = "Arc::new(RwLock::new(VecDeque::with_capacity(self.window_size.unwrap_or(SAMPLE_BUFFER_SIZE))))",
      setter(custom))]
    sample_buffer: Arc<RwLock<VecDeque<f32>>>,

    // The current index for inserting samples into the buffer
    #[builder(default = "AtomicUsize::new(0)", setter(skip))]
    insert_index: AtomicUsize,

    // The mix level of the effect 
    #[builder(default = "DEFAULT_MIX")]
    pub(crate) mix: f32,

    // Complement of mix, computed at build time
    #[builder(field(private), default = "1.0 - self.mix.unwrap_or(DEFAULT_MIX)")]
    mix_complement: f32,
}

impl Clone for Flanger {
    fn clone(&self) -> Self {
        Flanger {
            window_size: self.window_size,
            sample_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(self.window_size))),
            insert_index: AtomicUsize::new(self.insert_index.load(Ordering::SeqCst)),
            mix: self.mix,
            mix_complement: self.mix_complement,
        }
    }
}

impl PartialEq for Flanger {
    fn eq(&self, other: &Self) -> bool {
        self.window_size == other.window_size &&
        self.insert_index.load(Ordering::SeqCst) ==
            other.insert_index.load(Ordering::SeqCst) &&
        self.mix == other.mix &&
        self.mix_complement == other.mix_complement
    }
}

#[allow(dead_code)]
impl FlangerBuilder {
    pub(crate) fn sample_buffer(&mut self, sample_buffer: Arc<RwLock<VecDeque<f32>>>) -> &mut Self {
        let window_size = self.window_size.unwrap_or(DEFAULT_WINDOW_SIZE);
        for _ in 0..window_size {
            sample_buffer.write().unwrap().push_back(0.0);
        }
        self.sample_buffer = Some(sample_buffer);
        self
    }
}

#[allow(dead_code)]
impl Flanger {
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        let mut flanger_sample = 0.0;
        
        // Write new sample
        {
            let mut buffer = self.sample_buffer.write().unwrap();
            if buffer.len() < self.window_size {
                buffer.push_back(sample);
            } else {
                let idx = self.insert_index.load(Ordering::SeqCst) % self.window_size;
                if let Some(old_sample) = buffer.get_mut(idx) {
                    *old_sample = sample;
                }
            }
        }

        // Read delayed sample
        {
            let buffer = self.sample_buffer.read().unwrap();
            if !buffer.is_empty() {
                let read_idx = (self.insert_index.load(Ordering::SeqCst) + 1) % buffer.len();
                flanger_sample = *buffer.get(read_idx).unwrap_or(&0.0);
            }
        }

        // Update insert index
        self.insert_index.fetch_add(1, Ordering::SeqCst);

        // Mix original and flanged samples
        sample * self.mix_complement + flanger_sample * self.mix
    }
}

#[allow(dead_code)]
pub(crate) fn default_flanger() -> Flanger {
    FlangerBuilder::default()
        .window_size(SAMPLE_BUFFER_SIZE)
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn no_op_flanger() -> Flanger {
    FlangerBuilder::default()
        .window_size(0)
        .build().unwrap()
}
