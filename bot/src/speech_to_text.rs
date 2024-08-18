pub mod speech_to_text {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use whisper_rs::{FullParams, WhisperContext};

    pub trait SpeechToText {
        /// Recognize the audio and return the text.
        ///
        /// # Arguments
        /// * `audio` - The audio data as a vector of f32 samples. Note, these are not
        ///    the bytes of the audio file, but the actual samples.
        fn recognize(&self, audio: &Vec<f32>) -> String;
    }


    pub struct WhisperSTT {
    }
    impl SpeechToText for WhisperSTT {
        fn recognize(&self, audio: &Vec<f32>) -> String {
            self.wav_to_text(audio).unwrap_or_else(|e| format!("Error: {}", e))
        }
    }

    impl WhisperSTT {
        pub fn wav_to_text(&self, wav_data: &Vec<f32>) -> Result<String, Box<dyn std::error::Error>> {
            // Create a temporary file to store the .wav data
            let f32_wav_data = wav_data.to_owned();

            // Load the Whisper model
            let ctx = WhisperContext::new("/home/sgzmd/code/voicebot/models/ggml-base.bin")?;

            // Set up the parameters
            let mut params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);

            // Run the model
            let mut state = ctx.create_state()?;
            state.full(params, &f32_wav_data)?;

            // Extract the text
            let num_segments = state.full_n_segments()?;
            let mut text = String::new();
            for i in 0..num_segments {
                text.push_str(&state.full_get_segment_text(i)?);
                text.push(' ');
            }

            Ok(text.trim().to_string())
        }
    }
}