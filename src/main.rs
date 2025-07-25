#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod qso;
mod settings;
mod wavelog;
mod udp;

use qso::QSO;
use settings::Settings;
use wavelog::send;
use udp::UdpListener;

use iced::widget::{Column, Container, Text, Scrollable, Row, Space, Tooltip};
use iced::{Color, Element, Length, Task};

/// Maximum number of QSO records to keep in memory
const MAX_LOG_LINES: usize = 500;

/// Main application state
/// Manages QSO records, status information, and settings
#[derive(Debug)]
struct RustWavelogGateApp {
    qso_records: Vec<(QSO, String)>, // QSO records with their upload status
    status_message: String,          // Current status message for status bar
    listen_info: String,             // Connection information display
    settings: Option<Settings>,      // Application configuration
}

/// Application messages for event handling
#[derive(Debug, Clone)]
enum Message {
    /// Settings have been loaded from configuration file
    SettingsLoaded(Result<Settings, String>),
    /// QSO has been processed and sent to Wavelog
    QSOReceived(QSO, String),
    /// Raw UDP message received from ham radio software
    UdpMessage(Vec<u8>),
}

impl RustWavelogGateApp {
    /// Create new application instance and start initial tasks
    /// Returns the app state and a task to load configuration
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            qso_records: Vec::new(),
            status_message: "Loading...".to_string(),
            listen_info: String::new(),
            settings: None,
        };
        
        let task = Task::perform(Self::load_settings(), Message::SettingsLoaded);
        (app, task)
    }

    /// Load application settings from configuration file
    async fn load_settings() -> Result<Settings, String> {
        Settings::load().map_err(|e| format!("Configuration loading failed: {}", e))
    }

    /// Start UDP listener and wait for one message
    /// Used for continuous listening by restarting after each message
    async fn start_udp_listener(host: String, port: u16) -> Vec<u8> {
        let listener = UdpListener::new(host, port);
        listener.listen_once().await.unwrap_or_default()
    }

    /// Add a new QSO record to the beginning of the list
    /// Maintains a maximum number of records by removing the oldest ones
    fn add_qso_record(&mut self, qso: QSO, status: String) {
        self.qso_records.insert(0, (qso, status)); // Insert at the beginning, new QSO at the top
        if self.qso_records.len() > MAX_LOG_LINES {
            self.qso_records.pop(); // Remove the last (oldest) record
        }
    }

    /// Convert status text to display format
    /// Maps "OK" to "OK", everything else to "Error"
    fn get_status_display(status: &str) -> &str {
        if status == "OK" {
            "OK"
        } else {
            "Error"
        }
    }

    /// Process QSO data received from UDP and send to Wavelog
    /// Parses ADIF format and creates tasks for each valid QSO
    fn process_qso_data(&self, data: &[u8], settings: &Settings) -> Task<Message> {
        let adif = match std::str::from_utf8(data) {
            Ok(adif) => adif,
            Err(_) => return self.restart_udp_listener(settings),
        };

        let qsos = QSO::from_adif(adif);
        let mut tasks = Vec::new();

        // Process each QSO record
        for qso in qsos {
            if self.is_valid_qso(&qso) {
                tasks.push(self.send_qso_task(qso, settings.clone()));
            }
        }

        // Restart UDP listener for next message
        tasks.push(self.restart_udp_listener(settings));
        Task::batch(tasks)
    }

    /// Validate if a QSO record has all required fields
    /// Checks for non-empty call sign, date, time, and band
    fn is_valid_qso(&self, qso: &QSO) -> bool {
        !qso.call.is_empty() && !qso.qso_date.is_empty() 
            && !qso.time_on.is_empty() && !qso.band.is_empty()
    }

    /// Create an async task to send QSO to Wavelog
    /// Returns success/error status for UI display
    fn send_qso_task(&self, qso: QSO, settings: Settings) -> Task<Message> {
        Task::perform(
            async move {
                let status = match send(&qso, &settings).await {
                    Ok(_) => "OK".to_string(),
                    Err(e) => format!("{}", e),
                };
                (qso, status)
            },
            |(qso, status)| Message::QSOReceived(qso, status),
        )
    }

    /// Restart UDP listener to wait for next message
    fn restart_udp_listener(&self, settings: &Settings) -> Task<Message> {
        Task::perform(
            Self::start_udp_listener(settings.server.host.clone(), settings.server.port),
            Message::UdpMessage,
        )
    }

    /// Main message dispatcher for handling app events
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SettingsLoaded(result) => self.handle_settings_loaded(result),
            Message::UdpMessage(data) => self.handle_udp_message(data),
            Message::QSOReceived(qso, status) => self.handle_qso_received(qso, status),
        }
    }

    /// Handle settings loaded event
    /// Updates UI with connection info and starts UDP listener
    fn handle_settings_loaded(&mut self, result: Result<Settings, String>) -> Task<Message> {
        match result {
            Ok(settings) => {
                self.listen_info = format!("Listen: {}:{} | Wavelog: {}", 
                    settings.server.host, settings.server.port, settings.wavelog.url);
                self.status_message = "Ready".to_string();
                
                let task = self.restart_udp_listener(&settings);
                self.settings = Some(settings);
                task
            }
            Err(e) => {
                self.status_message = format!("Config load failed: {}", e);
                self.listen_info = String::new();
                Task::none()
            }
        }
    }

    /// Handle incoming UDP message with QSO data
    fn handle_udp_message(&mut self, data: Vec<u8>) -> Task<Message> {
        match &self.settings {
            Some(settings) => self.process_qso_data(&data, settings),
            None => Task::none(),
        }
    }

    /// Handle completed QSO processing result
    /// Updates the QSO record list and status message
    fn handle_qso_received(&mut self, qso: QSO, status: String) -> Task<Message> {
        if !qso.call.is_empty() {
            self.add_qso_record(qso, status);
            self.status_message = "QSO processed".to_string();
        }
        Task::none()
    }

    /// Build the application's user interface
    /// Creates a table layout with QSO records, header, and status bar
    pub fn view(&self) -> Element<Message> {
        // Create sticky table header with column titles
        let sticky_header = Container::new(
            Row::new()
                .push(Text::new("Time").width(Length::Fixed(60.0)).size(14).color(Color::from_rgb(0.8, 0.8, 0.8)))
                .push(Text::new("Call").width(Length::Fixed(120.0)).size(14).color(Color::from_rgb(0.8, 0.8, 0.8)))
                .push(Text::new("Grid").width(Length::Fixed(60.0)).size(14).color(Color::from_rgb(0.8, 0.8, 0.8)))
                .push(Text::new("Band").width(Length::Fixed(50.0)).size(14).color(Color::from_rgb(0.8, 0.8, 0.8)))
                .push(Text::new("Mode").width(Length::Fixed(50.0)).size(14).color(Color::from_rgb(0.8, 0.8, 0.8)))
                .push(Text::new("RST").width(Length::Fixed(64.0)).size(14).color(Color::from_rgb(0.8, 0.8, 0.8)))
                .push(Text::new("Status").width(Length::Fixed(50.0)).size(14).color(Color::from_rgb(0.8, 0.8, 0.8)))
                .padding(10)
                .spacing(5)
        )
        .style(|_| {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                border: iced::Border {
                    color: Color::from_rgb(0.4, 0.4, 0.4),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fill);

        // Create scrollable table rows for QSO data
        let mut data_rows = Vec::new();
        
        for (qso, status) in &self.qso_records {
            // Set status color: green for OK, red for errors
            let status_color = if status == "OK" { 
                Color::from_rgb(0.0, 0.8, 0.0) 
            } else { 
                Color::from_rgb(0.8, 0.0, 0.0) 
            };

            let status_display = Self::get_status_display(status);
            
            // Create status element with tooltip for error details
            let status_element: Element<Message> = if status == "OK" {
                Text::new(status_display).width(Length::Fixed(100.0)).size(12).color(status_color).font(iced::Font::MONOSPACE).into()
            } else {
                Tooltip::new(
                    Text::new(status_display).width(Length::Fixed(100.0)).size(12).color(status_color).font(iced::Font::MONOSPACE),
                    Text::new(status.as_str()).size(11),
                    iced::widget::tooltip::Position::Top
                )
                .style(|_theme| {
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.9))),
                        border: iced::Border {
                            color: Color::from_rgb(0.5, 0.5, 0.5),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        text_color: Some(Color::WHITE),
                        shadow: iced::Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 4.0,
                        },
                    }
                })
                .into()
            };

            let row = Container::new(
                Row::new()
                    .push(Text::new(&qso.time_on).width(Length::Fixed(60.0)).size(12).color(Color::WHITE).font(iced::Font::MONOSPACE))
                    .push(Text::new(&qso.call).width(Length::Fixed(120.0)).size(12).color(Color::WHITE).font(iced::Font::MONOSPACE))
                    .push(Text::new(&qso.gridsquare).width(Length::Fixed(60.0)).size(12).color(Color::WHITE).font(iced::Font::MONOSPACE))
                    .push(Text::new(&qso.band).width(Length::Fixed(50.0)).size(12).color(Color::WHITE).font(iced::Font::MONOSPACE))
                    .push(Text::new(&qso.mode).width(Length::Fixed(50.0)).size(12).color(Color::WHITE).font(iced::Font::MONOSPACE))
                    .push(Text::new(format!("{}/{}", qso.rst_sent, qso.rst_rcvd)).width(Length::Fixed(64.0)).size(12).color(Color::WHITE).font(iced::Font::MONOSPACE))
                    .push(status_element)
                    .padding(10)
                    .spacing(5)
            )
            .style(|_| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.12))),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            })
            .width(Length::Fill);
            
            data_rows.push(row.into());
        }

        // Create scrollable content for data rows only
        let scrollable_content: Element<Message> = if self.qso_records.is_empty() {
            // When empty, show centered message
            Container::new(
                Text::new("No QSO records").size(14).color(Color::from_rgb(0.6, 0.6, 0.6))
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.12))),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
        } else {
            // When has data, show scrollable list
            Scrollable::new(
                Column::with_children(data_rows)
                    .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        // Create status bar with border
        let status_bar = Container::new(
            Row::new()
                .push(Text::new(&self.listen_info).size(12).color(Color::from_rgb(0.7, 0.7, 0.7)))
                .push(Space::with_width(Length::Fill))
                .push(Text::new(&self.status_message).size(12).color(Color::from_rgb(0.9, 0.9, 0.9)))
                .padding(8)
                .width(Length::Fill)
        )
        .style(|_| {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.08))),
                border: iced::Border {
                    color: Color::from_rgb(0.4, 0.4, 0.4),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fill);

        // Main layout: sticky header + scrollable data + status bar
        let main_content = Column::new()
            .push(sticky_header)
            .push(scrollable_content)   
            .push(status_bar)
            .width(Length::Fill)
            .height(Length::Fill);

        Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                    ..Default::default()
                }
            })
            .into()
    }
}

impl Default for RustWavelogGateApp {
    fn default() -> Self {
        Self {
            qso_records: Vec::new(),
            status_message: "Starting...".to_string(),
            listen_info: String::new(),
            settings: None,
        }
    }
}

/// Application entry point
/// Sets up the GUI window with dark theme and loads the application icon
fn main() -> iced::Result {
    // Load and convert the application icon from embedded PNG data
    let icon = {
        let icon_data = include_bytes!("../icon.png");
        match image::load_from_memory(icon_data) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                iced::window::icon::from_rgba(rgba.into_raw(), width, height).ok()
            }
            Err(_) => None,
        }
    };

    // Configure and start the Iced application
    iced::application(
        |_state: &RustWavelogGateApp| format!("Wavelog Gate v{}", env!("CARGO_PKG_VERSION")),
        RustWavelogGateApp::update,
        RustWavelogGateApp::view
    )
        .theme(|_state| iced::Theme::Dark)
        .window(iced::window::Settings {
            size: iced::Size {
                width: 520.0,
                height: 240.0,
            },
            min_size: Some(iced::Size {
                width: 520.0,
                height: 240.0,
            }),
            resizable: true,
            decorations: true,
            icon,
            ..Default::default()
        })
        .run_with(RustWavelogGateApp::new)
}
