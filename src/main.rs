use config::Config;
use serde::{Serialize, Deserialize};
use tokio::net::UdpSocket;
use regex::Regex;
use std::collections::HashMap;
use reqwest::{Client, header};
use std::time::Duration;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub wavelog: WaveLogSettings,
    pub server: ServerSettings,
}

#[derive(Debug, Deserialize)]
pub struct WaveLogSettings {
    pub url: String,
    pub key: String,
    pub station: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String { String::from("0.0.0.0") }
fn default_port() -> u16 { 2333 }

#[derive(Debug, Default)]
pub struct QSO {
    pub call: String,
    pub gridsquare: String,
    pub mode: String,
    pub rst_sent: String,
    pub rst_rcvd: String,
    pub qso_date: String,
    pub time_on: String,
    pub qso_date_off: String,
    pub time_off: String,
    pub band: String,
    pub freq: String,
    pub freq_rx: String,
    pub operator: String,
    pub comment: String,
    pub power: String,
    pub my_gridsquare: String,
    pub station_callsign: String,
}

// 将 HashMap 转换为 QSO 结构体
fn create_qso(map: &HashMap<String, String>) -> QSO {
    QSO {
        call: map.get("call").cloned().unwrap_or_default(),
        gridsquare: map.get("gridsquare").cloned().unwrap_or_default(),
        mode: map.get("mode").cloned().unwrap_or_default(),
        rst_sent: map.get("rst_sent").cloned().unwrap_or_default(),
        rst_rcvd: map.get("rst_rcvd").cloned().unwrap_or_default(),
        qso_date: map.get("qso_date").cloned().unwrap_or_default(),
        time_on: map.get("time_on").cloned().unwrap_or_default(),
        qso_date_off: map.get("qso_date_off").cloned().unwrap_or_default(),
        time_off: map.get("time_off").cloned().unwrap_or_default(),
        band: map.get("band").cloned().unwrap_or_default(),
        freq: map.get("freq").cloned().unwrap_or_default(),
        freq_rx: map.get("freq_rx").cloned().unwrap_or_default(),
        operator: map.get("operator").cloned().unwrap_or_default(),
        comment: map.get("comment").cloned().unwrap_or_default(),
        power: map.get("power").cloned().unwrap_or_default(),
        my_gridsquare: map.get("my_gridsquare").cloned().unwrap_or_default(),
        station_callsign: map.get("station_callsign").cloned().unwrap_or_default(),
    }
}

pub fn adif2qso(input: &str) -> Vec<QSO> {
    let mut qsos = Vec::new();
    let mut current = HashMap::new();
    // 字段匹配正则表达式（忽略大小写）
    let re = Regex::new(r"(?i)<([a-z_]+)(?::(\d+))?(?::([a-z]+))?>([^<]*)").unwrap();
    for cap in re.captures_iter(input) {
        let field = cap[1].to_uppercase();
        let value = cap[4].trim().to_string();
        // 处理记录结束标记
        if field == "EOR" {
            qsos.push(create_qso(&current));
            current.clear();
            continue;
        }
        // 跳过头部相关标记
        if field == "EOH" {
            current.clear();
            continue;
        }
        // 存储字段值（自动转小写键名）
        current.insert(field.to_lowercase(), value);
    }
    qsos
}

fn qsos2adif(qso: &QSO) -> String {
    let mut adif = String::new();    
    if !qso.call.is_empty() { adif.push_str(&format!("<CALL:{}>{}", qso.call.len(), qso.call)); }
    if !qso.gridsquare.is_empty() { adif.push_str(&format!("<GRIDSQUARE:{}>{}", qso.gridsquare.len(), qso.gridsquare)); }

    if !qso.mode.is_empty() { adif.push_str(&format!("<MODE:{}>{}", qso.mode.len(), qso.mode)); }
    if !qso.rst_sent.is_empty() { adif.push_str(&format!("<RST_SENT:{}>{}", qso.rst_sent.len(), qso.rst_sent)); }
    if !qso.rst_rcvd.is_empty() { adif.push_str(&format!("<RST_RCVD:{}>{}", qso.rst_rcvd.len(), qso.rst_rcvd)); }
    if !qso.qso_date.is_empty() { adif.push_str(&format!("<QSO_DATE:{}>{}", qso.qso_date.len(), qso.qso_date)); }
    if !qso.time_on.is_empty() { adif.push_str(&format!("<TIME_ON:{}>{}", qso.time_on.len(), qso.time_on)); }

    if !qso.qso_date_off.is_empty() { adif.push_str(&format!("<QSO_DATE_OFF:{}>{}", qso.qso_date_off.len(), qso.qso_date_off)); }
    if !qso.time_off.is_empty() { adif.push_str(&format!("<TIME_OFF:{}>{}", qso.time_off.len(), qso.time_off)); }

    if !qso.band.is_empty() { adif.push_str(&format!("<BAND:{}>{}", qso.band.len(), qso.band)); }

    if !qso.freq.is_empty() { adif.push_str(&format!("<FREQ:{}>{}", qso.freq.len(), qso.freq)); }
    if !qso.freq_rx.is_empty() { adif.push_str(&format!("<FREQ_RX:{}>{}", qso.freq_rx.len(), qso.freq_rx)); }
    if !qso.operator.is_empty() { adif.push_str(&format!("<OPERATOR:{}>{}", qso.operator.len(), qso.operator)); }
    if !qso.comment.is_empty() { adif.push_str(&format!("<COMMENT:{}>{}", qso.comment.len(), qso.comment)); }

    if !qso.power.is_empty() { adif.push_str(&format!("<POWER:{}>{}", qso.power.len(), qso.power)); }

    if !qso.my_gridsquare.is_empty() { adif.push_str(&format!("<MY_GRIDSQUARE:{}>{}", qso.my_gridsquare.len(), qso.my_gridsquare)); }
    if !qso.station_callsign.is_empty() { adif.push_str(&format!("<STATION_CALLSIGN:{}>{}", qso.station_callsign.len(), qso.station_callsign)); }
    
    // End of record
    adif.push_str("<EOR>\r\n");
    
    adif
}

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
        string: qsos2adif(qso),
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
    let settings = Config::builder()
        // 添加配置文件（支持多种格式：INI, JSON, TOML 等）
        .add_source(config::File::with_name("config"))
        .build()?
        // 反序列化为结构体
        .try_deserialize::<Settings>()?;
    
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let sock = UdpSocket::bind(&addr).await?;
    println!("Listening on {}", addr);

    tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            if let Ok((len, _src)) = sock.recv_from(&mut buf).await {
                let msg = &buf[..len];
                if let Ok(adif) = std::str::from_utf8(msg) {
                    let qsos = adif2qso(adif);
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