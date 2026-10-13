use teloxide::prelude::*;
use teloxide::types::ParseMode;
use std::env;
use chrono::Utc;
mod models;
use models::MatchstatResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = Bot::from_env();

    if env::var("RUN_CRON").unwrap_or_default() == "true" {
        let chat_id = env::var("TELEGRAM_CHAT_ID")?.parse::<i64>()?;

        let summary = match fetch_tennis_scores().await {
            Ok(text) => text,
            Err(e) => format!("⚠️ Failed to fetch tennis scores: `{}`", e),
        };

        bot.send_message(ChatId(chat_id), summary)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        println!("Daily summary sent successfully via GitHub Actions!");
    }

    Ok(())
}

async fn fetch_tennis_scores() -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("TENNIS_API_KEY")?;
    
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let url_atp = format!("https://tennis-api-atp-wta-itf.p.rapidapi.com/tennis/v2/atp/results/{}", today);
    let url_wta = format!("https://tennis-api-atp-wta-itf.p.rapidapi.com/tennis/v2/wta/results/{}", today);

    let client = reqwest::Client::new();
    
    let (res_atp, res_wta) = tokio::try_join!(
        client.get(&url_atp)
            .header("X-RapidAPI-Key", &api_key)
            .header("X-RapidAPI-Host", "tennis-api-atp-wta-itf.p.rapidapi.com")
            .header("accept", "application/json")
            .send(),
        client.get(&url_wta)
            .header("X-RapidAPI-Key", &api_key)
            .header("X-RapidAPI-Host", "tennis-api-atp-wta-itf.p.rapidapi.com")
            .header("accept", "application/json")
            .send()
    )?;

    let response_text = res_atp.text().await?;
    println!("Raw API Response: {}", response_text);

    let atp_data: MatchstatResponse = serde_json::from_str(&response_text)?;
    let wta_data: MatchstatResponse = res_wta.json().await?;

    let mut message = format!("🎾 *Tennis Results for {}*\n\n", today);

    if atp_data.data.is_empty() && wta_data.data.is_empty() {
        message.push_str("_No matches found for today across ATP or WTA\\._");
        return Ok(message);
    }

    if !atp_data.data.is_empty() {
        message.push_str("🏆 *ATP Matches*\n");
        for m in atp_data.data.iter().take(5) {
            let tournament = m.tournament.name.clone();
            let p1 = m.player1.name.clone();
            let p2 = m.player2.name.clone();
            let score = m.result.clone().unwrap_or_else(|| "Score pending".into());

            let line = format!(
                "• *{}* — {} d\\. {} __{}__\n",
                escape_markdown(&tournament),
                escape_markdown(&p1),
                escape_markdown(&p2),
                escape_markdown(&score)
            );
            message.push_str(&line);
        }
        message.push_str("\n");
    }

    if !wta_data.data.is_empty() {
        message.push_str("🏆 *WTA Matches*\n");
        for m in wta_data.data.iter().take(5) {
            let tournament = m.tournament.name.clone();
            let p1 = m.player1.name.clone();
            let p2 = m.player2.name.clone();
            let score = m.result.clone().unwrap_or_else(|| "Score pending".into());

            let line = format!(
                "• *{}* — {} d\\. {} __{}__\n",
                escape_markdown(&tournament),
                escape_markdown(&p1),
                escape_markdown(&p2),
                escape_markdown(&score)
            );
            message.push_str(&line);
        }
    }

    Ok(message)
}

fn escape_markdown(input: &str) -> String {
    let chars = ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'];
    let mut escaped = String::new();
    for c in input.chars() {
        if chars.contains(&c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}