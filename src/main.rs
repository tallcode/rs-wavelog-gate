#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod qso;
mod settings;
mod wavelog;
mod udp;

use qso::QSO;
use settings::Settings;
use wavelog::send;
use udp::UdpListener;

use iced::widget::{Column, Container, Text, Scrollable};
use iced::{Color, Element, Length, Task};

const MAX_LOG_LINES: usize = 50;

#[derive(Debug)]
struct RustWavelogGateApp {
    lines: Vec<String>,
    settings: Option<Settings>,
}

#[derive(Debug, Clone)]
enum Message {
    SettingsLoaded(Result<Settings, String>),
    QSOReceived(QSO, String),
    UdpMessage(Vec<u8>),
}

impl RustWavelogGateApp {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            lines: vec!["Loading...".to_string()],
            settings: None,
        };
        
        let task = Task::perform(Self::load_settings(), Message::SettingsLoaded);
        (app, task)
    }

    async fn load_settings() -> Result<Settings, String> {
        Settings::load().map_err(|e| format!("Configuration loading failed: {}", e))
    }

    async fn start_udp_listener(host: String, port: u16) -> Vec<u8> {
        let listener = UdpListener::new(host, port);
        listener.listen_once().await.unwrap_or_default()
    }

    fn add_log_line(&mut self, line: String) {
        self.lines.push(line);
        if self.lines.len() > MAX_LOG_LINES {
            self.lines.remove(0);
        }
    }

    fn format_qso_log(&self, qso: &QSO, status: &str) -> String {
        if !qso.gridsquare.is_empty() {
            format!(
                "{} {} ({}) on {} (R:{} / S:{}) - {}", 
                qso.time_on, qso.call, qso.gridsquare, qso.band, qso.rst_sent, qso.rst_rcvd, status
            )
        } else {
            format!(
                "{} {} on {} (R:{} / S:{}) - {}", 
                qso.time_on, qso.call, qso.band, qso.rst_sent, qso.rst_rcvd, status
            )
        }
    }

    fn process_qso_data(&self, data: &[u8], settings: &Settings) -> Task<Message> {
        let adif = match std::str::from_utf8(data) {
            Ok(adif) => adif,
            Err(_) => return self.restart_udp_listener(settings),
        };

        let qsos = QSO::from_adif(adif);
        let mut tasks = Vec::new();

        // 处理每个 QSO
        for qso in qsos {
            if self.is_valid_qso(&qso) {
                tasks.push(self.send_qso_task(qso, settings.clone()));
            }
        }

        // 重启 UDP 监听
        tasks.push(self.restart_udp_listener(settings));
        Task::batch(tasks)
    }

    fn is_valid_qso(&self, qso: &QSO) -> bool {
        !qso.call.is_empty() && !qso.qso_date.is_empty() 
            && !qso.time_on.is_empty() && !qso.band.is_empty()
    }

    fn send_qso_task(&self, qso: QSO, settings: Settings) -> Task<Message> {
        Task::perform(
            async move {
                let status = match send(&qso, &settings).await {
                    Ok(_) => "OK".to_string(),
                    Err(e) => format!("ERROR {}", e),
                };
                (qso, status)
            },
            |(qso, status)| Message::QSOReceived(qso, status),
        )
    }

    fn restart_udp_listener(&self, settings: &Settings) -> Task<Message> {
        Task::perform(
            Self::start_udp_listener(settings.server.host.clone(), settings.server.port),
            Message::UdpMessage,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SettingsLoaded(result) => self.handle_settings_loaded(result),
            Message::UdpMessage(data) => self.handle_udp_message(data),
            Message::QSOReceived(qso, status) => self.handle_qso_received(qso, status),
        }
    }

    fn handle_settings_loaded(&mut self, result: Result<Settings, String>) -> Task<Message> {
        match result {
            Ok(settings) => {
                self.lines.clear();
                self.lines.push(format!("Listen: {}:{}", settings.server.host, settings.server.port));
                self.lines.push(format!("Wavelog Server: {}", settings.wavelog.url));
                
                let task = self.restart_udp_listener(&settings);
                self.settings = Some(settings);
                task
            }
            Err(e) => {
                self.lines.clear();
                self.lines.push(format!("Error: {}", e));
                Task::none()
            }
        }
    }

    fn handle_udp_message(&mut self, data: Vec<u8>) -> Task<Message> {
        match &self.settings {
            Some(settings) => self.process_qso_data(&data, settings),
            None => Task::none(),
        }
    }

    fn handle_qso_received(&mut self, qso: QSO, status: String) -> Task<Message> {
        if !qso.call.is_empty() {
            let log_line = self.format_qso_log(&qso, &status);
            self.add_log_line(log_line);
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let content = Column::with_children(
            self.lines
                .iter()
                .map(|line| {
                    Text::new(line)
                        .size(14)
                        .line_height(1.5)
                        .color(Color::WHITE)
                        .font(iced::Font::MONOSPACE)
                        .into()
                })
                .collect::<Vec<_>>(),
        )
        .padding(iced::Padding {
            top: 0.0,
            bottom: 0.0,
            left: 10.0,
            right: 10.0,
        })
        .width(Length::Fill);

        let scrollable = Scrollable::new(content).anchor_bottom();

        Container::new(scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Default for RustWavelogGateApp {
    fn default() -> Self {
        Self {
            lines: vec!["Starting...".to_string()],
            settings: None,
        }
    }
}

fn main() -> iced::Result {
    iced::application("Rust Wavelog Gate", RustWavelogGateApp::update, RustWavelogGateApp::view)
        .window(iced::window::Settings {
            size: iced::Size {
                width: 600.0,
                height: 200.0,
            },
            min_size: Some(iced::Size {
                width: 400.0,
                height: 150.0,
            }),
            resizable: true,
            decorations: true,
            ..Default::default()
        })
        .run_with(RustWavelogGateApp::new)
}
