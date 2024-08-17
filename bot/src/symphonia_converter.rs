use hound::{WavSpec, WavWriter};
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType as InterpolationType, WindowFunction};
use std::error::Error;
use std::io::Cursor;
use symphonia::core::audio::{SampleBuffer, SignalSpec};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub mod audio_conversion {
    use super::*;

    pub trait AudioConverter {
        fn convert_audio_to_wav(&self, input_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    }

    pub struct SymphoniaConverter;

    impl AudioConverter for SymphoniaConverter {
        fn convert_audio_to_wav(&self, input_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
            let my_input_data = input_data.to_owned();

            // Create a cursor for the input data
            let cursor = Cursor::new(my_input_data);

            // Create a media source stream
            let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

            // Create a hint to help the format registry guess what format reader is appropriate
            let hint = Hint::new();

            // Use the default options for metadata and format
            let format_opts: FormatOptions = Default::default();
            let metadata_opts: MetadataOptions = Default::default();

            // Probe the media source stream for a format
            let probed = symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

            // Get the format reader
            let mut format = probed.format;

            // Get the default track
            let track = format
                .default_track()
                .ok_or("No default track found in the audio data")?;

            // Create a decoder for the track
            let mut decoder = symphonia::default::get_codecs()
                .make(&track.codec_params, &DecoderOptions::default())?;

            // Retrieve the original sample rate
            let original_sample_rate = track.codec_params.sample_rate.unwrap();
            let num_channels = track.codec_params.channels.unwrap().count();

            // Create a WAV spec for 16 kHz mono
            let spec = WavSpec {
                channels: 1, // Mono
                sample_rate: 16000, // 16 kHz
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };

            // Create a buffer for the WAV data
            let mut wav_buffer = Cursor::new(Vec::new());
            {
                let mut writer = WavWriter::new(&mut wav_buffer, spec)?;

                // Set up the resampler with appropriate parameters
                let resample_ratio = 16000.0 / original_sample_rate as f64;
                let max_resample_ratio_relative = 10.0; // Set maximum resampling ratio, e.g., 10.0
                let sinc_len = 64; // Length of the sinc interpolation kernel
                let oversampling_factor = 128; // Oversampling factor for the resampler
                let chunk_size = 1024; // Size of input data in frames

                let params = SincInterpolationParameters {
                    sinc_len,
                    f_cutoff: 0.95, // Cutoff frequency
                    interpolation: InterpolationType::Linear, // Interpolation type
                    oversampling_factor,
                    window: WindowFunction::BlackmanHarris2, // Window function
                };

                let mut resampler = SincFixedIn::<f64>::new(
                    resample_ratio,
                    max_resample_ratio_relative,
                    params,
                    chunk_size,
                    1, // Mono output
                )?;

                // Decode and process packets
                loop {
                    let packet = match format.next_packet() {
                        Ok(packet) => packet,
                        Err(SymphoniaError::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                            // End of stream reached
                            break;
                        }
                        Err(SymphoniaError::ResetRequired) => {
                            // Reset the decoder
                            decoder.reset();
                            continue;
                        }
                        Err(e) => {
                            // Handle other errors
                            return Err(Box::new(e));
                        }
                    };

                    // Decode the packet
                    let decoded = decoder.decode(&packet)?;

                    // Create a sample buffer
                    let mut sample_buffer = SampleBuffer::<i16>::new(decoded.capacity() as u64, *decoded.spec());
                    sample_buffer.copy_interleaved_ref(decoded);

                    // Mix to mono if needed
                    let samples = sample_buffer.samples();
                    let mut mono_samples: Vec<f64> = Vec::with_capacity(samples.len() / num_channels);

                    for i in (0..samples.len()).step_by(num_channels) {
                        // Mix to mono by averaging channels
                        let mut mono_sample = 0.0;
                        for c in 0..num_channels {
                            mono_sample += samples[i + c] as f64;
                        }
                        mono_sample /= num_channels as f64;
                        mono_samples.push(mono_sample);
                    }

                    // Resample to 16 kHz
                    let resampled_samples = resampler.process(&[mono_samples], None)?;

                    // Write the resampled mono samples to the WAV buffer
                    for &sample in resampled_samples[0].iter() {
                        writer.write_sample(sample as i16)?;
                    }
                }
            }

            Ok(wav_buffer.into_inner())
        }
    }
}
