use std::io::Write;
use std::process::Command as ProcessCommand;
use tempfile::NamedTempFile;
use which::which;

use voicebot::audio_conversion::audio_conversion::convert_audio_to_wav;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Helper function to create a sample audio file
    fn create_sample_audio(format: &str) -> Vec<u8> {
        let output = ProcessCommand::new("ffmpeg")
            .args(&[
                "-f", "lavfi", 
                "-i", "sine=frequency=1000:duration=5",
                "-acodec", format,
                "-f", format,
                "pipe:1"
            ])
            .output()
            .expect("Failed to create sample audio");

        output.stdout
    }

    #[test]
    fn test_convert_mp3_to_wav() {
        let mp3_data = create_sample_audio("mp3");
        let result = convert_audio_to_wav(&mp3_data);
        assert!(result.is_ok());
        let wav_data = result.unwrap();
        assert!(!wav_data.is_empty());
        assert!(wav_data.starts_with(b"RIFF"));
    }

    #[test]
    fn test_convert_ogg_to_wav() {
        let ogg_data = create_sample_audio("ogg");
        let result = convert_audio_to_wav(&ogg_data);
        assert!(result.is_ok());
        let wav_data = result.unwrap();
        assert!(!wav_data.is_empty());
        assert!(wav_data.starts_with(b"RIFF"));
    }

    #[test]
    fn test_convert_wav_to_wav() {
        let wav_data = create_sample_audio("wav");
        let result = convert_audio_to_wav(&wav_data);
        assert!(result.is_ok());
        let converted_wav_data = result.unwrap();
        assert!(!converted_wav_data.is_empty());
        assert!(converted_wav_data.starts_with(b"RIFF"));
    }

    #[test]
    fn test_convert_empty_input() {
        let empty_data = vec![];
        let result = convert_audio_to_wav(&empty_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_invalid_audio() {
        let invalid_data = b"This is not audio data".to_vec();
        let result = convert_audio_to_wav(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_output_file_properties() {
        let mp3_data = create_sample_audio("mp3");
        let result = convert_audio_to_wav(&mp3_data);
        assert!(result.is_ok());
        let wav_data = result.unwrap();

        // Save the WAV data to a temporary file
        let mut temp_wav = NamedTempFile::new().unwrap();
        temp_wav.write_all(&wav_data).unwrap();

        // Use ffprobe to check the audio properties
        let output = ProcessCommand::new("ffprobe")
            .args(&[
                "-v", "quiet",
                "-print_format", "json",
                "-show_format",
                "-show_streams",
                temp_wav.path().to_str().unwrap()
            ])
            .output()
            .expect("Failed to run ffprobe");

        let probe_output = String::from_utf8_lossy(&output.stdout);

        println!("probe_output={}", probe_output);

        probe_output.contains("codec_name");

        assert!(probe_output.contains("\"codec_name\": \"pcm_s16le\""));
        assert!(probe_output.contains("\"sample_rate\": \"44100\""));
        assert!(probe_output.contains("\"channels\": 2"));
    }

    #[test]
    fn test_large_file_conversion() {
        let large_audio = create_sample_audio("mp3");
        let large_audio = large_audio.repeat(10);  // Make the file 10 times larger
        let result = convert_audio_to_wav(&large_audio);
        assert!(result.is_ok());
        let wav_data = result.unwrap();
        assert!(!wav_data.is_empty());
        assert!(wav_data.starts_with(b"RIFF"));
    }

    #[test]
    fn test_ffmpeg_not_installed() {
        // Temporarily rename ffmpeg to simulate it not being installed
        let ffmpeg_path = which::which("ffmpeg").unwrap();
        let temp_path = ffmpeg_path.with_file_name("ffmpeg_temp");
        fs::rename(&ffmpeg_path, &temp_path).unwrap();

        let mp3_data = create_sample_audio("mp3");
        let result = convert_audio_to_wav(&mp3_data);
        assert!(result.is_err());

        // Restore ffmpeg
        fs::rename(&temp_path, &ffmpeg_path).unwrap();
    }
}