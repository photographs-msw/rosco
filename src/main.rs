extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// static NUM_ARGS: usize = 3;
// static NUM_CHANNELS: u16 = 1;
// static BITS_PER_SAMPLE: u16 = 1;
// static DURATION: Lazy<Option<f32>> = Lazy::new(|| {
//     env::var("DURATION").ok().unwrap().parse().ok()
// });
// static SAMPLE_RATE: Lazy<Option<f32>> = Lazy::new(|| {
//     env::var("SAMPLE_RATE").ok().unwrap().parse().ok()
// });
// static FREQUENCY: Lazy<Option<f32>> = Lazy::new(|| {
//     env::var("FREQUENCY").ok().unwrap().parse().ok()
// });
// static DYNAMIC_ARRAY: Lazy<Mutex<Vec<f32>>> = Lazy::new(|| {
//     let mut vec: Vec<f32> = Vec::new();
//     for i in 0..(DURATION.unwrap() * SAMPLE_RATE.unwrap()) as usize {
//         let t = i as f32 / SAMPLE_RATE.unwrap();
//         vec.push((2.0 * std::f32::consts::PI * FREQUENCY.unwrap() * t).sin());
//     }
//     Mutex::new(vec)
// });

fn main() {
    // let args = get_args();
    // let sample_rate: f32 = args[0].parse().unwrap();
    // let frequency: f32= args[1].parse().unwrap();
    // let duration: f32 = args[2].parse().unwrap();
    // if args.len() != NUM_ARGS {
    //     eprintln!("This program requires exactly {} arguments.", NUM_ARGS);
    //     std::process::exit(1);
    // }

    // let locked_array= DYNAMIC_ARRAY.lock().unwrap().clone();
    // let samples
    //     = Arc::new(Mutex::new(DYNAMIC_ARRAY.lock().unwrap().clone().iter()));

    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    run::<f32>(&device, &config.into());
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig)
          // samples: Arc<Mutex<Iter<'static, T>>>)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    // Produce a sinusoid of maximum amplitude.
    let sample_rate = 44100.0;
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);

    let stream =
        device.build_output_stream(config,
                                   move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                                       write_data::<T>(data, channels, &mut next_value)
                                   },
                                   err_fn,
                                   None).unwrap();
    stream.play().unwrap();

    // Keep the stream alive indefinitely.
    // std::thread::sleep(std::time::Duration::from_millis(10000));
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
                 // samples: Arc<Mutex<Iter<T>>>)
where T: cpal::Sample + cpal::FromSample<f32>, {
    // let samples
    //     = Arc::new(Mutex::new(DYNAMIC_ARRAY.lock().unwrap().clone().into_iter()));
    // let sample_rate = config.sample_rate.0 as f32;
    // let mut samples_to_write= samples.lock().unwrap();
    for output_frame in output.chunks_mut(channels) {
        let sample_to_write = next_sample();
        let value = T::from_sample::<f32>(sample_to_write);
        // let value: T = <dyn cpal::Sample<Float=(), Signed=()>>::from(&value);
        for output_sample in output_frame.iter_mut() {
            *output_sample = value;
        }
    }
}

// fn get_args() -> Vec<String> {
//     let args: Vec<String> = env::args().skip(1).collect();
//     return args;
// }

    // let spec = generate_wav_spec(sample_rate.parse().unwrap());
    // generate_sine_wave_file(
    //     spec,
    //     sample_rate.parse().unwrap(),
    //     frequency.parse().unwrap(),
    //     duration.parse().unwrap(),
    //     amplitude_factor.parse().unwrap(),
    // );
// }

// fn generate_wav_spec(sample_rate: f32) -> WavSpec {
//     let spec = hound::WavSpec {
//         channels: NUM_CHANNELS,
//         sample_rate: sample_rate as u32,
//         bits_per_sample: BITS_PER_SAMPLE,
//         sample_format: hound::SampleFormat::Int,
//     };
//
//     return spec;
// }
//
// fn generate_sine_wave_file(spec: WavSpec, sample_rate: f32, frequency: f32, duration: f32,
//                            amplitude_factor: f32) {
//     let mut writer =
//         hound::WavWriter::create("sine_wave.wav", spec).unwrap();
//
//     let samples = (0..(duration * sample_rate) as usize)
//         .map(|i| {
//             let t = i as f32 / sample_rate;
//             let amplitude = amplitude_factor * i16::MAX as f32;
//             (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()) as i16
//         });
//
//     for sample in samples {
//         writer.write_sample(sample).unwrap();
//     }
//
//     writer.finalize().unwrap();
// }
//
