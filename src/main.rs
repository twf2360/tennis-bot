use teloxide::prelude::*;
use teloxide::types::ParseMode;
use std::env;
use chrono::Utc;
use serde::Deserialize;

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

#[derive(Deserialize, Debug)]
struct MatchstatResponse {
    result: Vec<MatchItem>,
}

#[derive(Deserialize, Debug)]
struct MatchItem {
    tournament_name: Option<String>,
    player1: Option<String>,
    player2: Option<String>,
    score: Option<String>,
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
            .send(),
        client.get(&url_wta)
            .header("X-RapidAPI-Key", &api_key)
            .header("X-RapidAPI-Host", "tennis-api-atp-wta-itf.p.rapidapi.com")
            .send()
    )?;

    let atp_data: MatchstatResponse = res_atp.json().await?;
    let wta_data: MatchstatResponse = res_wta.json().await?;

    let mut message = format!("🎾 *Tennis Results for {}*\n\n", today);

    if atp_data.result.is_empty() && wta_data.result.is_empty() {
        message.push_str("_No matches found for today across ATP or WTA\\._");
        return Ok(message);
    }

    if !atp_data.result.is_empty() {
        message.push_str("🏆 *ATP Matches*\n");
        for m in atp_data.result.iter().take(5) {
            let tournament = m.tournament_name.clone().unwrap_or_else(|| "ATP Tour".into());
            let p1 = m.player1.clone().unwrap_or_else(|| "Player 1".into());
            let p2 = m.player2.clone().unwrap_or_else(|| "Player 2".into());
            let score = m.score.clone().unwrap_or_else(|| "Score pending".into());

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

    if !wta_data.result.is_empty() {
        message.push_str("🏆 *WTA Matches*\n");
        for m in wta_data.result.iter().take(5) {
            let tournament = m.tournament_name.clone().unwrap_or_else(|| "WTA Tour".into());
            let p1 = m.player1.clone().unwrap_or_else(|| "Player 1".into());
            let p2 = m.player2.clone().unwrap_or_else(|| "Player 2".into());
            let score = m.score.clone().unwrap_or_else(|| "Score pending".into());

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