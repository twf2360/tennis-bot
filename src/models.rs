use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchstatResponse {
    pub data: Vec<MatchItem>,
    pub page: u32,
    pub page_no: u32,
    pub page_size: u32,
    pub limit: u32,
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchItem {
    pub id: String,
    pub match_id: String,
    pub date: String,
    pub round_id: u32,
    pub player1_id: u64,
    pub player2_id: u64,
    pub tournament_id: u64,
    pub match_winner: Option<u64>,
    pub result: Option<String>,
    pub result_type: Option<String>,
    pub best_of: Option<u32>,
    pub odd1: Option<String>,
    pub odd2: Option<String>,
    pub player1: Player,
    pub player2: Player,
    pub tournament: Tournament,
    pub odds_available: bool,
    pub links: Option<Links>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub country_acr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tournament {
    pub id: u64,
    pub name: String,
    pub court_id: Option<u32>,
    pub date: Option<String>,
    pub rank_id: Option<u32>,
    pub country_acr: Option<String>,
    pub court: Option<String>,
    pub rank: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Links {
    pub odds: Option<String>,
}