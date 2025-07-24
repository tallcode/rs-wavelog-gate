use crate::qso::QSO;
use crate::settings::Settings;

use serde::Serialize;
use reqwest::{Client, header};
use std::time::Duration;
use std::error::Error;


#[derive(Serialize)]

struct WavelogPayload {
    key: String,
    station_profile_id: String,
    #[serde(rename = "type")]
    type_field: String,
    string: String,

}

pub async fn send(qso: &QSO,  settings: &Settings) -> Result<String, Box<dyn Error>> {
    // Create a client with the appropriate settings
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()?;
    // Prepare the payload
    let payload = WavelogPayload {
        key: settings.wavelog.key.trim().to_string(),
        station_profile_id: settings.wavelog.station.trim().to_string(),
        type_field: "adif".to_string(),
        string: qso.to_adif(),
    };
    // Convert payload to JSON
    let post_data = serde_json::to_string(&payload)?;
    // Prepare the URL
    let url = format!("{}/api/qso", settings.wavelog.url);
    // Get the version from Cargo.toml or provide a default
    let version = option_env!("CARGO_PKG_VERSION").unwrap_or("1.0");

    
    // Send the request
    let response = client
        .post(&url)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::USER_AGENT, format!("RustClient_v{}", version))
        .body(post_data)
        .send()
        .await?;

    
    // Get the status code
    let status_code = response.status();
    // Get the response body
    let res_string = response.text().await?;
    // Check if request was successful
    if !status_code.is_success() {
        if res_string.contains("html>") {
            return Err("Wrong URL".into());
        }
        return Err(res_string.into());
    }
    Ok(res_string)
}