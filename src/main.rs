mod qso;
mod settings;

use qso::QSO;
use settings::Settings;

use serde::Serialize;
use tokio::net::UdpSocket;
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

pub async fn send_to_wavelog(qso: &QSO,  settings: &Settings) -> Result<String, Box<dyn Error>> {
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::load()?;
    
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let sock = UdpSocket::bind(&addr).await?;
    println!("Listening on {}", addr);

    tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            if let Ok((len, _src)) = sock.recv_from(&mut buf).await {
                let msg = &buf[..len];
                if let Ok(adif) = std::str::from_utf8(msg) {
                    let qsos = QSO::from_adif(adif);
                    for qso in qsos {
                        // 发送到 Wavelog
                        if !qso.call.is_empty() && !qso.qso_date.is_empty() && !qso.time_on.is_empty() && !qso.band.is_empty() {
                            let status = match send_to_wavelog(&qso, &settings).await {
                                Ok(_res) => "OK".to_string(),
                                Err(_e) => "ERROR".to_string()
                            };
                            // print log like 080415 RQ6Z (KN98) on 10m (R:-24 / S:-15) - OK
                            if !qso.gridsquare.is_empty() {
                                println!("{} {} ({}) on {} (R:{} / S:{}) - {}", qso.time_on, qso.call, qso.gridsquare, qso.band, qso.rst_sent, qso.rst_rcvd, status);
                            } else {
                                println!("{} {} on {} (R:{} / S:{}) - {}", qso.time_on, qso.call, qso.band, qso.rst_sent, qso.rst_rcvd, status);
                            }
                        }
                    }
                }
            }
        }
    });

    // 保持主线程运行
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}