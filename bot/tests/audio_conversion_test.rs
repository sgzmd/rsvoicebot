#[cfg(test)]
mod tests {
    use super::*;
    use hound::{WavSpec, WavWriter, SampleFormat};
    use std::io::Cursor;
    use voicebot::audio_conversion::audio_conversion::convert_wav_to_samples;

    #[test]
    fn test_convert_wav_to_samples() {
        // Generate a simple sine wave as test data
        let sample_rate = 44100;
        let duration_seconds = 1;
        let frequency = 440.0;
        let amplitude = 0.5;
        let num_samples = sample_rate * duration_seconds;

        // Define WAV file specifications
        let spec = WavSpec {
            channels: 1,
            sample_rate: sample_rate as u32,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        // Create a buffer to hold the WAV data
        let mut buffer = Vec::new();
        {
            // Use a Cursor to wrap the Vec<u8> buffer
            let mut cursor = Cursor::new(&mut buffer);
            let mut writer = WavWriter::new(&mut cursor, spec).unwrap();
            for i in 0..num_samples {
                let sample = (amplitude * (2.0 * std::f64::consts::PI * frequency * i as f64 / sample_rate as f64).sin() * i16::MAX as f64) as i16;
                writer.write_sample(sample).unwrap();
            }
            writer.finalize().unwrap(); // Make sure to finalize the writer to properly write the WAV headers
        }

        // Now, `buffer` contains the WAV file bytes
        // Convert the WAV bytes to samples using the function under test
        let result = convert_wav_to_samples(&buffer);
        let audio_data = result.expect("Failed to convert WAV to samples");
        let samples = audio_data.samples;

        // Ensure that the samples vector is not empty
        assert!(!samples.is_empty(), "The samples vector is empty");

        // Check that the length of the samples vector matches the expected number of samples
        assert_eq!(samples.len(), num_samples, "The number of samples does not match the expected value");

        // Check the contents of the samples (e.g., check the first few samples)
        // For a sine wave, we expect the values to oscillate between -1.0 and 1.0
        assert!(samples.iter().all(|&s| s >= -1.0 && s <= 1.0), "Samples are out of expected range");

        assert_eq!(audio_data.duration, duration_seconds as f64, "The duration of the audio data does not match the expected value");
    }
}
