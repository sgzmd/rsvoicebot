pub mod audio_conversion {
    use std::error::Error;
    use std::process::{Command, Stdio};
    use std::io::{Write, Read};
    use std::fs::File;
    use tempfile::NamedTempFile;
    use crate::audio_conversion::audio_conversion::AudioConverter;

    pub struct FFMpegAudioConverter;

    impl AudioConverter for FFMpegAudioConverter {
        fn convert_audio_to_wav(&self, input_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
            // Create a temporary file to store the input data
            let mut input_file = NamedTempFile::new()?;
            input_file.write_all(input_data)?;
            let input_path = input_file
                .path()
                .to_str()
                .ok_or_else(|| "Invalid input file path")?;

            Self::convert_file_to_wav(input_path)
        }
    }

    impl FFMpegAudioConverter {
        pub fn convert_file_to_wav(input_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
            // Create another temporary file to store the output WAV data
            let output_file = NamedTempFile::new()?;
            let output_path = output_file.path().to_str().ok_or("Invalid output file path")?;

            // Run FFmpeg command to convert input file to WAV
            let status = Command::new("ffmpeg")
                .arg("-y")  // Overwrite output file if it exists
                .arg("-i")
                .arg(input_path) // Input file path
                .arg("-ar")
                .arg("16000") // Sample rate 16 kHz
                .arg("-ac")
                .arg("1") // 1 channels (mono)
                .arg("-f")
                .arg("wav") // Output format
                .arg("-acodec")
                .arg("pcm_s16le") // PCM signed 16-bit little-endian
                .arg(output_path) // Output file path
                .stderr(Stdio::null()) // Suppress FFmpeg output
                .stdout(Stdio::null())
                .status()?;

            // Check if the FFmpeg process completed successfully
            if !status.success() {
                return Err("FFmpeg conversion failed".into());
            }

            // Read the WAV data from the output file
            let mut output_wav = Vec::new();
            File::open(output_path)?.read_to_end(&mut output_wav)?;

            Ok(output_wav)
        }
    }
}
