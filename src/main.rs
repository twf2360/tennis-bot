use teloxide::prelude::*;
use teloxide::types::ParseMode;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = Bot::from_env();

    if env::var("RUN_CRON").unwrap_or_default() == "true" {
        let chat_id = env::var("TELEGRAM_CHAT_ID")?.parse::<i64>()?;

        let summary = "🎾 *Daily Tennis Round\\-Up*\n\n\
            this is a test message!";

        bot.send_message(ChatId(chat_id), summary)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        println!("Daily summary sent successfully via GitHub Actions!");
    }

    Ok(())
}