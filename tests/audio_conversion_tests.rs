use std::io::Write;
use std::process::Command as ProcessCommand;
use tempfile::NamedTempFile;
use which::which;

use voicebot::ffmpeg_converter::audio_ffmpeg_converter::FfmpegConverter;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // TODO: Add tests for the audio conversion
}