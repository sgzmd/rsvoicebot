use teloxide::{net::Download, prelude::*, types::BotCommand, utils::command::BotCommands};
use voicebot::audio_conversion::audio_conversion::AudioConverter;
use std::error::Error;
use teloxide::types::Currency::AUD;
use voicebot::symphonia_converter::symphonia_converter::SymphoniaConverter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    Command::repl(bot, answer).await;

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
    match cmd {
        Command::Help => help(bot, msg).await?,
        Command::Recognize => recognize(bot, msg).await?,
        Command::Summarize => summarize(bot, msg).await?,
    }

    Ok(())
}

async fn help(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn recognize(bot: Bot, msg: Message) -> ResponseResult<()> {
    let text = msg.text().unwrap_or("No text provided");
    let response = format!("Recognized text: {}", text);

    if let Some(audio) = msg.audio() {
        let file = audio.file.clone();
        let file_id = file.unique_id;

        // Download the audio file
        let file = bot.get_file(file_id).await?;

        let mut buffer: Vec<u8> = Vec::new();
        bot.download_file(&file.path, &mut buffer).await?;

        let converter = SymphoniaConverter;
        let wavbytes = converter.convert_audio_to_wav(buffer.as_slice());

        if wavbytes.is_ok() {
            log::info!(
                "Successfully converted audio to wav, {} bytes read",
                wavbytes.unwrap().len()
            );
        }
    }

    bot.send_message(msg.chat.id, response).await?;

    Ok(())
}

async fn summarize(bot: Bot, msg: Message) -> ResponseResult<()> {
    let text = msg.text().unwrap_or("No text provided");
    let summary = format!("Summary of: {}", text); // Implement actual summarization logic here

    bot.send_message(msg.chat.id, summary).await?;

    Ok(())
}


