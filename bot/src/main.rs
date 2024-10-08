use std::env;
use std::error::Error;
use std::fs::File;
use std::time::Instant;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::types::Currency::AUD;
use teloxide::{net::Download, prelude::*, types::BotCommand, utils::command::BotCommands};
use tempfile::tempdir;
use voicebot::audio_conversion::audio_conversion::convert_wav_to_samples;
use voicebot::audio_conversion::audio_conversion::AudioConverter;
use voicebot::ffmpeg_converter::audio_conversion::FFMpegAudioConverter;
use voicebot::speech_to_text::speech_to_text::{SpeechToText, WhisperSTT};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    teloxide::repl(
        bot,
        |bot: Bot, msg: Message| async move {
            recognize(bot, msg).await?;
            Ok(())
        },
    ).await;

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "recognize the attached audio file.")]
    Recognize,
    #[command(description = "summarize the attached text and/or audio")]
    Summarize,
    #[command(description = "display this text.")]
    Help,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    recognize(bot, msg).await?;
    // match cmd {
    //     Command::Help => help(bot, msg).await?,
    //     Command::Recognize => recognize(bot, msg).await?,
    //     Command::Summarize => summarize(bot, msg).await?,
    // }

    Ok(())
}

async fn help(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn recognize(bot: Bot, msg: Message) -> ResponseResult<()> {
    let mut file_id : Option<String> = None;

    if let Some(voice) = msg.voice() {
        log::info!("This is a voice message");
        file_id = Some(voice.clone().file.id);

    } else if let Some(audio) = msg.audio() {
        log::info!("Generic audio file attached to the message");
        file_id = Some(audio.clone().file.id);
    }

    if let Some(fid) = file_id {
        let file = bot.get_file(fid).await?;

        let mut buffer: Vec<u8> = Vec::new();
        bot.download_file(&file.path, &mut buffer).await?;

        let converter = FFMpegAudioConverter;

        // FIXME: replace unwrap with better error propagation
        let wav_bytes = converter
            .convert_audio_to_wav(buffer.as_slice()).unwrap();

        // FIXME: replace unwrap with better error propagation
        let audio_data = convert_wav_to_samples(wav_bytes.as_slice()).unwrap();
        let samples = audio_data.samples;

        let total_seconds = audio_data.duration.round() as u32; // Round to nearest second and convert to u32
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;

        let ratio: f64 = env::var("RECORDING_TO_WALL_RATIO")
            .unwrap_or("10".to_string())  // Use 10 as a default value if the env variable is not set
            .parse()                      // Parse the string to a floating-point number
            .unwrap_or(10.0);             // Use 10 as a default value if parsing fails

        let expected_time = (total_seconds as f64 / ratio) as u64;

        let expected_minutes = expected_time / 60;
        let expected_seconds = expected_time % 60;

        let mut expected_time_str : String;
        if expected_minutes > 0 {
            expected_time_str = format!("{} minutes {} seconds", expected_minutes, expected_seconds);
        } else {
            expected_time_str = format!("{} seconds", expected_seconds);
        }

        bot.send_message(
            msg.chat.id,
            format!(
                "Audio duration: {} minutes {} seconds.\nExpected recognition time: {}",
                minutes,
                seconds,
                expected_time_str),

        )
            .await?;

        let stt = WhisperSTT::new(Option::None).unwrap();

        let start_time = Instant::now();
        let recognized_text = stt.recognize(&samples);
        let recognition_duration = start_time.elapsed().as_secs_f64();

        log::info!("Recognized text: {}", recognized_text);
        // Let's say 100 seconds for 200 seconds of recording
        // then we can say we recognise 2 seconds of recording in one second
        // i.e. 2 seconds of recording in 1 second of real time
        let real_time_duration = total_seconds as f64 / recognition_duration;
        // send log message with this information
        log::info!("Recognition speed: {} seconds of audio in second", real_time_duration);

        // send telegram message with this info
        bot.send_message(msg.chat.id, format!("Actual recognition speed: {} seconds of audio in second", real_time_duration)).await?;

        if recognized_text.len() > 4096 {
            let dir  = tempdir()?;
            let path = dir.path().join("recognized_text.txt");
            std::fs::write(&path, recognized_text)?;

            // Send the file as an attachment
            bot.send_document(msg.chat.id,
                              teloxide::types::InputFile::file(path))
                .await?;
        } else {
            bot.send_message(msg.chat.id, recognized_text).await?;
        }
    } else {
        bot.send_message(msg.chat.id, "Something went wrong").await?;
    }


    Ok(())
}

async fn summarize(bot: Bot, msg: Message) -> ResponseResult<()> {
    let text = msg.text().unwrap_or("No text provided");
    let summary = format!("Summary of: {}", text); // Implement actual summarization logic here

    bot.send_message(msg.chat.id, summary).await?;

    Ok(())
}


