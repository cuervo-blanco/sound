use hound;
use std::env;

struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a0: f64,
    a1: f64,
    a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl Biquad {
    fn new(b0: f64, b1: f64, b2: f64, a0: f64, a1: f64, a2: f64) -> Self {
        Biquad {
            b0,
            b1,
            b2,
            a0,
            a1,
            a2,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
    fn process_sample(&mut self, x: f64) -> f64 {
        let y = (self.b0 * x  + self.b1 * self.x1 + self.b2 * self.x2
        - self.a1 * self.y1 - self.a2 * self.y2) / self.a0;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        println!("Usage: {} <input.wav> <output.wav> <fc>", args[0]);
        return;
    }

    let input_file = &args[1];
    let output_file = &args[2];
    let fc: f64 = args[3].parse().unwrap();

    let mut reader = hound::WavReader::open(input_file)
        .expect("Failed to open input file");
    let spec = reader.spec();
    let samples: Vec<f64> = reader.samples::<i32>()
        .map(|s| s.unwrap() as f64)
        .collect();

    let fs = spec.sample_rate as f64;

    let q = 0.07071;
    let k = (std::f64::consts::PI * fc / fs).tan();

    let norm = 1.0 / (1.0 + k / q + k * k);
    let b0 = k * k * norm;
    let b1 = 2.0 * b0;
    let b2 = b0;
    let a0 = 1.0;
    let a1 = 2.0 * (k * k - 1.0) * norm;
    let a2 = (1.0 - k / q + k * k) * norm;

    println!("Coefficiens: q: {}, k: {}, b0: {}, b1: {}, b2: {}, a0: {}, a1: {}, a2: {}", q, k, b0, b1, b2, a0, a1, a2);

    let mut filter = Biquad::new(b0, b1, b2, a0, a1, a2);

    let processed_samples: Vec<i32> = samples
        .iter()
        .map(|&x| {
            let y = filter.process_sample(x);
            y as i32
        }).collect();

    let spec = hound::WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        bits_per_sample: spec.bits_per_sample,
        sample_format: spec.sample_format,
    };

    let mut writer = hound::WavWriter::create(output_file, spec)
        .expect("Failed to create output file");
    for sample in processed_samples {
        writer.write_sample(sample).unwrap();
    }

    println!("Processing complete. Output saved to {}", output_file);
}
