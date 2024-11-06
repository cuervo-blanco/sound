use clap::{Parser, CommandFactory};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::process::exit;

#[derive(Parser)]
#[command(
    author = "cuervo-blanco",
    version = "0.1.0",
    about = "A simple command-line audio player.",
    long_about = None
)]
struct Cli {
    #[arg(value_name = "AUDIO_FILE")]
    audio_file: String,
}

fn main() {
    let args = Cli::parse();

    let file = match File::open(&args.audio_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file '{}': {}", args.audio_file, e);
            Cli::command().print_help().unwrap();
            println!();
            exit(1);
        }
    };

    let buf_reader = BufReader::new(file);
    let source = match Decoder::new(buf_reader) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Error decoding audio file '{}': {}", args.audio_file, e
            );
            exit(1);
        }
    };

    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error getting default output stream: {}", e);
            exit(1);
        }
    };

    let sink = match Sink::try_new(&stream_handle) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error creating audio sink: {}", e);
            exit(1);
        }
    };

    sink.append(source);
    sink.sleep_until_end();

}
