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
    let url_atp = format!("https://tennis-api-atp-wta-itf.p.rapidapi.com/tennis/v2/atp/results/{}?pageSize=50", today);
    let url_wta = format!("https://tennis-api-atp-wta-itf.p.rapidapi.com/tennis/v2/wta/results/{}?pageSize=50", today);

    let client = reqwest::Client::new();
    
    let res_atp = client.get(&url_atp)
        .header("X-RapidAPI-Key", &api_key)
        .header("X-RapidAPI-Host", "tennis-api-atp-wta-itf.p.rapidapi.com")
        .send()
        .await?;


    if res_atp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Ok("⚠️ *Rate limit hit on ATP endpoint*. Waiting for reset\\.".into());
    }

    let atp_text = res_atp.text().await?;
    let atp_data: MatchstatResponse = serde_json::from_str(&atp_text)?;

    // pause briefly to respect per-second rate limits
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    let res_wta = client.get(&url_wta)
        .header("X-RapidAPI-Key", &api_key)
        .header("X-RapidAPI-Host", "tennis-api-atp-wta-itf.p.rapidapi.com")
        .send()
        .await?;

    if res_wta.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Ok("⚠️ *Rate limit hit on WTA endpoint*. Waiting for reset\\.".into());
    }

    let wta_text = res_wta.text().await?;
    let wta_data: MatchstatResponse = serde_json::from_str(&wta_text)?;

    let mut message = escape_markdown("🎾 *Tennis Results for today\n\n*");

    if atp_data.data.is_empty() && wta_data.data.is_empty() {
        message.push_str("_No matches found for today across ATP or WTA._");
        return Ok(message);
    }

    if !atp_data.data.is_empty() {
        message.push_str("🏆 *ATP Matches*\n");
        let mut count = 0;

        for m in atp_data.data.iter() {
            let tournament = m.tournament.name.clone();

            if !is_desired_tier(&tournament) {
                continue;
            }

            let score = m.result.clone().unwrap_or_else(|| "Score pending".into());

            let winner_name;
            let loser_name;

            if Some(m.player1.id) == m.match_winner {
                winner_name = m.player1.name.clone();
                loser_name = m.player2.name.clone();
            } else if Some(m.player2.id) == m.match_winner {
                winner_name = m.player2.name.clone();
                loser_name = m.player1.name.clone();
            } else {
                winner_name = m.player1.name.clone();
                loser_name = m.player2.name.clone();
            }

            let line = format!(
                "• *{}* — *{}* d\\. {} __{}__\n",
                escape_markdown(&tournament),
                escape_markdown(&winner_name),
                escape_markdown(&loser_name),
                escape_markdown(&score)
            );
            message.push_str(&line);

            count += 1;
            if count >= 5 {
                message.push_str("\n\n");
                break;
            }
        }
    }

    if !wta_data.data.is_empty() {
        message.push_str("🏆 *WTA Matches*\n");
        let mut count = 0;

        for m in wta_data.data.iter() {
            let tournament = m.tournament.name.clone();

            if !is_desired_tier(&tournament) {
                continue;
            }

            let score = m.result.clone().unwrap_or_else(|| "Score pending".into());

            let winner_name;
            let loser_name;

            if Some(m.player1.id) == m.match_winner {
                winner_name = m.player1.name.clone();
                loser_name = m.player2.name.clone();
            } else if Some(m.player2.id) == m.match_winner {
                winner_name = m.player2.name.clone();
                loser_name = m.player1.name.clone();
            } else {
                winner_name = m.player1.name.clone();
                loser_name = m.player2.name.clone();
            }

            let line = format!(
                "• *{}* — *{}* d\\. {} __{}__\n",
                escape_markdown(&tournament),
                escape_markdown(&winner_name),
                escape_markdown(&loser_name),
                escape_markdown(&score)
            );
            message.push_str(&line);

            count += 1;
            if count >= 5 {
                break;
            }
        }
    }

    Ok(message)
}

fn escape_markdown(input: &str) -> String {
    let chars = ['-','_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'];
    let mut escaped = String::new();
    for c in input.chars() {
        if chars.contains(&c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

fn is_desired_tier(tournament_name: &str) -> bool {
    let lower = tournament_name.to_lowercase();
    
    lower.contains("grand slam")
        || lower.contains("masters")
        || lower.contains("1000")
        || lower.contains("500")
}