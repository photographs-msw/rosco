use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use circular_queue::CircularQueue;

const BUFFER_SIZE: usize = 44100; // 1 second at 44.1kHz
const NUM_SAMPLES: usize = 44100 * 10; // 10 seconds worth of samples

// Struct mimicking the VecDeque implementation
struct VecDequeBuffer {
    buffer: Arc<RwLock<VecDeque<f32>>>,
    write_index: usize,
    read_index: usize,
}

impl VecDequeBuffer {
    fn new(size: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(size))),
            write_index: 0,
            read_index: 0,
        }
    }

    fn write(&mut self, sample: f32) {
        let mut buffer = self.buffer.write().unwrap();
        if buffer.len() < BUFFER_SIZE {
            buffer.push_back(sample);
        } else {
            buffer[self.write_index % BUFFER_SIZE] = sample;
        }
        self.write_index += 1;
    }

    fn read(&mut self) -> f32 {
        let buffer = self.buffer.read().unwrap();
        if buffer.is_empty() {
            return 0.0;
        }
        let sample = buffer[self.read_index % buffer.len()];
        self.read_index += 1;
        sample
    }
}

// Struct using CircularQueue
struct CircularBuffer {
    buffer: Arc<RwLock<CircularQueue<f32>>>,
}

impl CircularBuffer {
    fn new(size: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(CircularQueue::with_capacity(size))),
        }
    }

    fn write(&mut self, sample: f32) {
        let mut buffer = self.buffer.write().unwrap();
        buffer.push(sample);
    }

    fn read(&mut self) -> f32 {
        let buffer = self.buffer.read().unwrap();
        buffer.iter().next().copied().unwrap_or(0.0)
    }
}

fn bench_vecdeque(c: &mut Criterion) {
    let mut buffer = VecDequeBuffer::new(BUFFER_SIZE);
    
    c.bench_function("vecdeque_write", |b| {
        b.iter(|| {
            for i in 0..NUM_SAMPLES {
                buffer.write(black_box(i as f32));
            }
        })
    });

    c.bench_function("vecdeque_read", |b| {
        b.iter(|| {
            for _ in 0..NUM_SAMPLES {
                black_box(buffer.read());
            }
        })
    });

    // Benchmark interleaved read/write
    c.bench_function("vecdeque_interleaved", |b| {
        b.iter(|| {
            for i in 0..NUM_SAMPLES {
                buffer.write(black_box(i as f32));
                black_box(buffer.read());
            }
        })
    });
}

fn bench_circular(c: &mut Criterion) {
    let mut buffer = CircularBuffer::new(BUFFER_SIZE);
    
    c.bench_function("circular_write", |b| {
        b.iter(|| {
            for i in 0..NUM_SAMPLES {
                buffer.write(black_box(i as f32));
            }
        })
    });

    c.bench_function("circular_read", |b| {
        b.iter(|| {
            for _ in 0..NUM_SAMPLES {
                black_box(buffer.read());
            }
        })
    });

    // Benchmark interleaved read/write
    c.bench_function("circular_interleaved", |b| {
        b.iter(|| {
            for i in 0..NUM_SAMPLES {
                buffer.write(black_box(i as f32));
                black_box(buffer.read());
            }
        })
    });
}

criterion_group!(benches, bench_vecdeque, bench_circular);
criterion_main!(benches); 