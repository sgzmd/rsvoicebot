pub mod audio_conversion {
    use std::error::Error;

    // AudioConverter trait with a single method to convert audio data to WAV format.
    // It returns bytes of the converted WAV file (pcm,_s16le, 44100 Hz), not the samples.
    pub trait AudioConverter {
        fn convert_audio_to_wav(&self, input_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    }
}