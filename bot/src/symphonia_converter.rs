use hound::{WavSpec, WavWriter};
use std::error::Error;
use std::io::Cursor;
use symphonia::core::audio::SampleBuffer;
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

            // Create the WAV spec
            let spec = WavSpec {
                channels: track.codec_params.channels.unwrap().count() as u16,
                sample_rate: track.codec_params.sample_rate.unwrap(),
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };

            // Create a buffer for the WAV data
            let mut wav_buffer = Cursor::new(Vec::new());
            {
                let mut writer = WavWriter::new(&mut wav_buffer, spec)?;

                // Decode and write packets
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

                    // Write the samples to the WAV buffer
                    for &sample in sample_buffer.samples() {
                        writer.write_sample(sample)?;
                    }
                }
            }

            Ok(wav_buffer.into_inner())
        }
    }
}