pub mod audio_conversion {
    use std::error::Error;

    pub trait AudioConverter {
        fn convert_audio_to_wav(&self, input_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    }
}