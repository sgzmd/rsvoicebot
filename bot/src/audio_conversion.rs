pub mod audio_conversion {
    use std::io::{Cursor, BufReader};
    use hound::{WavReader, SampleFormat};
    use std::error::Error;

    pub struct AudioData {
        pub samples: Vec<f32>,
        pub duration: f64, // Duration in seconds
    }

    // AudioConverter trait with a single method to convert audio data to WAV format.
    // It returns bytes of the converted WAV file (pcm,_s16le, 44100 Hz), not the samples.
    pub trait AudioConverter {
        fn convert_audio_to_wav(&self, input_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    }

    pub fn convert_wav_to_samples(wav_bytes: &[u8]) -> Result<AudioData, Box<dyn Error>> {
        // Create a cursor for the input bytes
        let cursor = Cursor::new(wav_bytes);

        // Create a buffered reader for the cursor
        let reader = BufReader::new(cursor);

        // Initialize the WavReader
        let wav_reader = WavReader::new(reader)?;
        let mut wr = wav_reader;

        // Convert the WAV data to a Vec<f32> based on the sample format
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
                    _ => return Err("Unsupported bit depth".into()),
                }
            }
        };

        // Calculate the duration of the audio clip
        let sample_rate = wr.spec().sample_rate as usize;
        let num_channels = wr.spec().channels as usize;
        let num_samples = wav_data.len();

        let duration_seconds = num_samples as f64 / (sample_rate * num_channels) as f64;

        // Return an AudioData struct with the samples and duration
        Ok(AudioData {
            samples: wav_data,
            duration: duration_seconds,
        })
    }
}