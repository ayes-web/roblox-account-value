use gloo::net::http::Request;
use serde::{Deserialize, Serialize};

pub const API_URL: &str = "https://roblox-account-value-api.sly.ee";

#[derive(Serialize, Deserialize)]
pub struct CollectiblesAccountValue {
    pub total_robux: i64,
    pub in_euro: i64,
}

pub async fn collectibles_account_value(userid: u64) -> Result<CollectiblesAccountValue, gloo::net::Error> {
    let response = Request::get(&format!("{API_URL}/api/collectibles-account-value")).query([("userid", userid.to_string())]).send().await?;
    let response_body = response.text().await?;
    let collectibles_account_value: CollectiblesAccountValue = serde_json::from_str(&response_body).unwrap();
    Ok(collectibles_account_value)
}


pub async fn can_view_inventory(userid: u64) -> Result<bool, gloo::net::Error> {
    let response = Request::get(&format!("{API_URL}/api/can-view-inventory")).query([("userid", userid.to_string())]).send().await?;
    Ok(response.text().await?.parse::<bool>().unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct ProfileInfo {
    pub username: String,
    pub avatar: String,
}

pub async fn profile_info(userid: u64) -> Result<ProfileInfo, gloo::net::Error> {
    let response = Request::get(&format!("{API_URL}/api/profile-info")).query([("userid", userid.to_string())]).send().await?;
    let response_body = response.text().await?;
    let profile_info: ProfileInfo = serde_json::from_str(&response_body).unwrap();
    Ok(profile_info)
}


#[derive(Serialize, Deserialize)]
pub struct ExchangeRate {
    pub robux_per_euro: i64,
}

pub async fn exchange_rate() -> Result<ExchangeRate, gloo::net::Error> {
    let response = Request::get(&format!("{API_URL}/api/exchange-rate")).send().await?;
    let response_body = response.text().await?;
    let exchange_rate: ExchangeRate = serde_json::from_str(&response_body).unwrap();
    Ok(exchange_rate)
}