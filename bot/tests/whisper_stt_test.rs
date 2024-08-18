#[cfg(test)]
mod tests {
    use hound::{SampleFormat, WavReader};
    use std::fs::File;
    use std::io::BufReader;
    use voicebot::speech_to_text::speech_to_text::WhisperSTT;

    fn to_lowercase_and_remove_punctuation(input: &str) -> String {
        input
            .to_lowercase() // Convert to lowercase
            .chars() // Convert to an iterator over characters
            .filter(|c| c.is_alphanumeric() || c.is_whitespace()) // Keep only alphanumeric characters and whitespace
            .collect() // Collect the filtered characters into a String
    }

    fn test_stt(path: &str, expected: &str) {
        let file = File::open(path).expect("Failed to open test file");

        let reader = BufReader::new(file);

        let wav_reader = WavReader::new(reader);
        let mut wr = wav_reader.unwrap();
        use hound::SampleFormat;

        let wav_data: Vec<f32> = match wr.spec().sample_format {
            SampleFormat::Float => wr.samples::<f32>().map(|s| s.unwrap()).collect(),
            SampleFormat::Int => {
                let bits_per_sample = wr.spec().bits_per_sample;
                match bits_per_sample {
                    8 => wr.samples::<i8>()
                        .map(|s| (s.unwrap() as f32) / 128.0)
                        .collect(),
                    16 => wr.samples::<i16>()
                        .map(|s| (s.unwrap() as f32) / 32768.0)
                        .collect(),
                    24 => wr.samples::<i32>()
                        .map(|s| (s.unwrap() as f32) / 8388608.0)
                        .collect(),
                    32 => wr.samples::<i32>()
                        .map(|s| (s.unwrap() as f32) / 2147483648.0)
                        .collect(),
                    _ => panic!("Unsupported bit depth"),
                }
            }
        };


        // Create an instance of WhisperSTT
        let whisper_stt = WhisperSTT {};

        // Perform the speech-to-text recognition
        let result = to_lowercase_and_remove_punctuation(&whisper_stt.wav_to_text(&wav_data).expect("STT failed"));

        // Assert the result matches the expected text
        assert_eq!(result, expected);
    }

    #[test]
    fn test_whisper_stt_wav_to_text_en() {
        // Load the test .wav file
        let path = "test_assets/golden.wav";
        let expected_text = to_lowercase_and_remove_punctuation("this is a test, this is just a test");

        test_stt(path, expected_text.as_str());
    }
}
