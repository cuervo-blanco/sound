use std::process;
use clap::{Arg, Command};

use hound;

fn main() {
    // Add in future possibilty of use convolving with a specific
    // transer function, that way the user can select different effects
    // such as filtering the sound to a specific frequency with a filter
    // of their chosen order
    // convolve signal1.wav filter -o 3 -t iir -f 345
    // and also reverbs such as:
    // convolve signal2.wav reverb -ir my_impulse_response.wav -rt 5s
    //
    let matches = Command::new("Convolve")
        .version("0.1.1")
        .author("cuervo-blanco")
        .about("Convolves two WAV audio signals")
        .arg(
            Arg::new("input1")
                .help("First input WAV file")
                .required(true)
                .index(1),
        ).arg(
            Arg::new("input2")
                .help("Second input WAV file")
                .required(true)
                .index(2),
        ).arg(
            Arg::new("output")
                .help("Output WAV file name")
                .long("name")
                .short('n')
                .default_value("output.wav"),
        ).get_matches();

    let input1_path = matches.get_one::<String>("input1").unwrap();
    let input2_path = matches.get_one::<String>("input2").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();
    let (mut signal1, mut spec1): (Vec<Vec<f32>>, hound::WavSpec) = read_wav_file(input1_path);
    let (mut signal2, mut spec2): (Vec<Vec<f32>>, hound::WavSpec) = read_wav_file(input2_path);

    if spec1.sample_rate != spec2.sample_rate {
        if spec1.sample_rate < spec2.sample_rate {
            println!(
                "Resampling '{}' from {} Hz to {} Hz",
                input1_path, spec1.sample_rate, spec2.sample_rate
            );
            signal1 = resample_signal(&signal1, spec1.sample_rate, spec2.sample_rate);
            spec1.sample_rate = spec2.sample_rate;
        } else {
            println!(
                "Resampling '{}' from {} Hz to {} Hz",
                input2_path, spec2.sample_rate, spec1.sample_rate
            );
            signal2 = resample_signal(&signal2, spec2.sample_rate, spec1.sample_rate);
            spec2.sample_rate = spec1.sample_rate;
        }
    }

    if spec1.channels != spec2.channels {
        eprintln!("Input WAV files must have the same number of channels.");
        process::exit(1);
    }
    let num_channels = spec1.channels as usize;
    let mut result_per_channel = Vec::new();
    for channel in 0..num_channels {
        let s1 = &signal1[channel];
        let s2 = &signal2[channel];

        let result = convolve(s1, s2);
        result_per_channel.push(result);
    }
    let output_spec = hound::WavSpec {
        channels: spec1.channels,
        sample_rate: spec1.sample_rate,
        bits_per_sample: spec1.bits_per_sample,
        sample_format: spec1.sample_format,
    };
    let output_name = format!("{}.wav", output_path);
    write_wav_file(&output_name, &result_per_channel, &output_spec);
    // Send to Python or C++ to generate graphs and such (data_analysis)
}
fn resample_signal(
    signal_per_channel: &[Vec<f32>],
    input_sample_rate: u32,
    output_sample_rate: u32,
) -> Vec<Vec<f32>> {
    let ratio = output_sample_rate as f64 / input_sample_rate as f64;
    let mut resampled_per_channel = Vec::new();

    for channel_samples in signal_per_channel {
        let input_len = channel_samples.len();
        let output_len = ((input_len as f64) * ratio) as usize;
        let mut resampled = Vec::with_capacity(output_len);

        for i in 0..output_len {
            let src_pos = i as f64 / ratio;
            let src_index = src_pos.floor() as usize;
            let frac = src_pos - src_index as f64;

            let s0 = channel_samples.get(src_index).cloned().unwrap_or(0.0);
            let s1 = channel_samples.get(src_index + 1).cloned().unwrap_or(0.0);
            let interpolated = s0 + (s1 - s0) * frac as f32;

            resampled.push(interpolated);
        }
        resampled_per_channel.push(resampled);
    }

    resampled_per_channel
}

fn read_wav_file(path: &str) -> (Vec<Vec<f32>>, hound::WavSpec) {
    let mut reader = hound::WavReader::open(path).expect("Failed to open WAV file");
    let spec = reader.spec();
    let num_channels = spec.channels as usize;
    if spec.bits_per_sample != 16 && spec.sample_format == hound::SampleFormat::Int {
        eprintln!(
            "Unsupported bits_per_sample: {}. Only 16-bit integer WAV files are supported.",
            spec.bits_per_sample
        );
        process::exit(1);
    }
    // Add capability for 24 bit integer?
    if spec.bits_per_sample != 32 && spec.sample_format == hound::SampleFormat::Float {
        eprintln!(
            "Unsupported bits_per_sample: {}. Only 32-bit float WAV files are supported.",
            spec.bits_per_sample
        );
        process::exit(1);
    }
    let mut samples_per_channel = vec![Vec::new(); num_channels];
    let mut channel_index = 0;
    match spec.sample_format {
        hound::SampleFormat::Int => {
            for sample in reader.samples::<i16>() {
                let sample = sample.unwrap() as f32 / i16::MAX as f32;
                samples_per_channel[channel_index].push(sample);
                channel_index = (channel_index + 1) % num_channels;
            }
        }
        hound::SampleFormat::Float => {
            for sample in reader.samples::<f32>() {
                let sample = sample.unwrap();
                samples_per_channel[channel_index].push(sample);
                channel_index = (channel_index + 1) % num_channels;
            }
        }
    };

    (samples_per_channel, spec)
}

fn write_wav_file(path: &str, samples_per_channel: &[Vec<f32>], spec: &hound::WavSpec) {
    let mut writer = hound::WavWriter::create(path, *spec).expect("Failed to create WAV file");
    let num_channels = spec.channels as usize;
    let num_samples = samples_per_channel
        .iter()
        .map(|channel_samples| channel_samples.len())
        .max()
        .unwrap_or(0);

    for i in 0..num_samples {
        for channel in 0..num_channels {
            let sample = if i < samples_per_channel[channel].len() {
                samples_per_channel[channel][i]
            } else {
                0.0
            };
            match spec.sample_format {
                hound::SampleFormat::Int => {
                    let s = (sample * i16::MAX as f32) as i16;
                    writer.write_sample(s).unwrap();
                }
                hound::SampleFormat::Float => {
                    writer.write_sample(sample).unwrap();
                }
            }
        }
    }
}

fn convolve(signal1: &[f32], signal2: &[f32]) -> Vec<f32> {
    let n = signal1.len();
    let m = signal2.len();
    let mut result = vec![0.0; n + m - 1];

    for i in 0..n {
        for j in 0..m {
            result[i + j] += signal1[i] * signal2[j];
        }
    }

    result
}

