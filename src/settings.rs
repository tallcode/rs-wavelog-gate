use serde::Deserialize;
use config::Config;

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

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?
            .try_deserialize::<Settings>()
    }
}