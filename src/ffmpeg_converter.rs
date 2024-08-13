pub mod audio_ffmpeg_converter {
    use crate::audio_conversion::audio_conversion::AudioConverter;

    pub struct FfmpegConverter;

    impl AudioConverter for FfmpegConverter {
        fn convert_audio_to_wav(&self, input_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Create temporary files for input and output
            let mut input_file = tempfile::NamedTempFile::new()?;
            let output_file = tempfile::NamedTempFile::new()?;
        
            // Write input data to the temporary input file
            input_file.write_all(input_data)?;
            input_file.flush()?;
        
            // Prepare ffmpeg command
            let status = ProcessCommand::new("ffmpeg")
                .arg("-i")
                .arg(input_file.path())
                .arg("-acodec")
                .arg("pcm_s16le")
                .arg("-ar")
                .arg("44100")
                .arg("-ac")
                .arg("2")
                .arg("-f")
                .arg("wav")
                .arg(output_file.path())
                .arg("-y")
                .status()?;
        
            // Check if ffmpeg command was successful
            if !status.success() {
                return Err("FFmpeg conversion failed".into());
            }
        
            // Read the output file into a buffer
            let output_data = std::fs::read(output_file.path())?;
        
            Ok(output_data)
        }
    }
}