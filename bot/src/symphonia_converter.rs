pub mod symphonia_converter {
    use crate::audio_conversion::audio_conversion::AudioConverter;
    use symphonia::core::audio::{SampleBuffer, SignalSpec};
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::{MediaSourceStream, ReadOnlySource};
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use symphonia::default::{get_codecs, get_probe};
    use std::error::Error;
    use std::io::{Cursor, Write};

    pub struct SymphoniaConverter;

    impl AudioConverter for SymphoniaConverter {
        fn convert_audio_to_wav(&self, input_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
            let data = input_data.clone().to_owned();

            // Create a cursor from input data to simulate a file
            let cursor = Cursor::new(data);

            // Create a media source stream using Box with the correct lifetime
            let mss = MediaSourceStream::new(Box::new(ReadOnlySource::new(cursor)), Default::default());

            // Create a probe hint to help format detection
            let mut hint = Hint::new();

            // Use the default probe to guess the format
            let mut probed = get_probe()
                .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;

            // Get the first track with audio in the format
            let track = probed
                .format
                .tracks()
                .iter()
                .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
                .ok_or("No audio track found")?
                .clone();

            let mut decoder = get_codecs()
                .make(&track.codec_params, &DecoderOptions::default())?;

            // Buffer to store the converted WAV data
            let mut wav_buffer = Vec::new();

            // WAV header creation
            let spec = SignalSpec {
                rate: track.codec_params.sample_rate.unwrap(),
                channels: track.codec_params.channels.unwrap(),
            };

            write_wav_header(&mut wav_buffer, spec)?;

            // Decode and convert the audio data
            let mut packet_buffer = None;

            loop {
                let packet = match probed.format.next_packet() {
                    Ok(packet) => packet,
                    Err(symphonia::core::errors::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(Box::new(e)),
                };

                if packet.track_id() == track.id {
                    let decoded = decoder.decode(&packet)?;

                    if packet_buffer.is_none() {
                        // Initialize packet_buffer with the correct capacity and spec
                        let spec = decoded.spec().clone();

                        packet_buffer = Some(SampleBuffer::new(decoded.capacity() as u64, spec));
                    }

                    if let Some(buffer) = &mut packet_buffer {
                        buffer.copy_interleaved_ref(decoded);
                        wav_buffer.write_all(buffer.samples())?;
                    }
                }
            }

            Ok(wav_buffer)
        }
    }

    fn write_wav_header<W: Write>(writer: &mut W, spec: SignalSpec) -> Result<(), Box<dyn Error>> {
        let channels = spec.channels.count() as u16;
        let sample_rate = spec.rate;
        let bits_per_sample = 16;
        let byte_rate = sample_rate * channels as u32 * (bits_per_sample / 8) as u32;
        let block_align = channels * (bits_per_sample / 8) as u16;

        writer.write_all(b"RIFF")?;
        writer.write_all(&0u32.to_le_bytes())?; // Placeholder for file size
        writer.write_all(b"WAVE")?;
        writer.write_all(b"fmt ")?;
        writer.write_all(&16u32.to_le_bytes())?;
        writer.write_all(&1u16.to_le_bytes())?; // PCM format
        writer.write_all(&channels.to_le_bytes())?;
        writer.write_all(&sample_rate.to_le_bytes())?;
        writer.write_all(&byte_rate.to_le_bytes())?;
        writer.write_all(&block_align.to_le_bytes())?;
        writer.write_all(&(bits_per_sample as u16).to_le_bytes())?;

        writer.write_all(b"data")?;
        writer.write_all(&0u32.to_le_bytes())?; // Placeholder for data chunk size

        Ok(())
    }
}
