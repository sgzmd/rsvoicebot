use std::error::Error;
use std::fs::File;
use teloxide::types::Currency::AUD;
use teloxide::{net::Download, prelude::*, types::BotCommand, utils::command::BotCommands};
use voicebot::audio_conversion::audio_conversion::{convert_wav_to_samples, AudioConverter};
use voicebot::ffmpeg_converter::audio_conversion::{AudioConverter as FFMpegAC, FFMpegAudioConverter};
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
        let wavbytes = converter
            .convert_audio_to_wav(buffer.as_slice())
            .unwrap();

        // FIXME: replace unwrap with better error propagation
        let samples = convert_wav_to_samples(wavbytes.as_slice()).unwrap();
        let stt = WhisperSTT {};
        let recognized_text = stt.recognize(&samples);

        log::info!("Recognized text: {}", recognized_text);

        bot.send_message(msg.chat.id, recognized_text).await?;
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


