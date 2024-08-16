#[cfg(test)]
mod tests {
    use std::fs;
    use voicebot::symphonia_converter::symphonia_converter::SymphoniaConverter;
    use voicebot::audio_conversion::audio_conversion::AudioConverter;

    #[test]
    fn test_mp3_to_wav_conversion() {
        // Load the test MP3 file
        let test_file_path = "test_assets/test.mp3";
        let input_data = fs::read(test_file_path).expect("Failed to read test MP3 file");

        // Create an instance of SymphoniaConverter
        let converter = SymphoniaConverter;

        // Convert the MP3 file to WAV format
        let result = converter.convert_audio_to_wav(&input_data);

        let wav_data = result.expect("Audio conversion failed");


        assert!(!wav_data.is_empty(), "Output WAV data is empty");

        // Optionally, write the WAV data to a file for manual inspection
        fs::write("test_assets/output.wav", &wav_data).expect("Failed to write output WAV file");
    }
}
