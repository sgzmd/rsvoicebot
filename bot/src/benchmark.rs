use std::env;
use std::error::Error;
use std::time::Instant;
use log::info;
use voicebot::audio_conversion::audio_conversion::{convert_wav_to_samples, AudioConverter};
use voicebot::ffmpeg_converter::audio_conversion::FFMpegAudioConverter;
use voicebot::speech_to_text::speech_to_text::{SpeechToText, WhisperSTT};

fn main() {
    pretty_env_logger::init();

    let args: Vec<String> = env::args().collect();

    // Check if we have enough arguments
    if args.len() < 3 {
        eprintln!("Usage: {} <model> <input>", args[0]);
        std::process::exit(1);
    }

    let model = &args[1];
    let input = &args[2];

    run_benchmark(input, model).unwrap()
}

fn run_benchmark(input: &str, model: &str) -> Result<(), Box<dyn Error>> {
    info!("Starting benchmark run");
    info!("Model: {}", model);
    info!("Input: {}", input);

    let conv = FFMpegAudioConverter;
    let bytes = FFMpegAudioConverter::convert_file_to_wav(input)?;

    let audio_data = convert_wav_to_samples(bytes.as_slice())?;
    let samples = audio_data.samples;

    let total_seconds = audio_data.duration.round() as u32; // Round to nearest second and convert to u32
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    info!("Audio duration: {} minutes, {} seconds ({} samples)",
        minutes,
        seconds,
        samples.len());

    let stt = WhisperSTT::new(Some(model))?;
    let start_time = Instant::now();
    let recognized_text = stt.recognize(&samples);
    let recognition_duration = start_time.elapsed().as_secs_f64();

    info!("Recognized text: {}", recognized_text);
    // Let's say 100 seconds for 200 seconds of recording
    // then we can say we recognise 2 seconds of recording in one second
    // i.e. 2 seconds of recording in 1 second of real time
    let real_time_duration = total_seconds as f64 / recognition_duration;
    // send log message with this information
    info!("Recognition speed: {} seconds of audio in second", real_time_duration);

    Ok(())
}
