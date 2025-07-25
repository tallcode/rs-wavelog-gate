use tokio::net::UdpSocket;
use std::time::Duration;

/// UDP listener for receiving ADIF data from Ham Radio software
#[derive(Debug)]
pub struct UdpListener {
    host: String,
    port: u16,
}

impl UdpListener {
    /// Create a new UDP listener instance
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    /// Start UDP server and listen for data
    /// Returns the content of the first received data packet
    pub async fn listen_once(&self) -> Result<Vec<u8>, UdpListenerError> {
        let addr = format!("{}:{}", self.host, self.port);
        
        let sock = UdpSocket::bind(&addr).await
            .map_err(|e| UdpListenerError::BindError(addr.clone(), e))?;

        let mut buf = [0; 4096]; // Increase buffer size to handle larger ADIF data
        
        loop {
            match sock.recv_from(&mut buf).await {
                Ok((len, _src)) => {
                    if len > 0 {
                        return Ok(buf[..len].to_vec());
                    }
                }
                Err(e) => {
                    // Log error but continue listening
                    eprintln!("UDP receive error: {}", e);
                    // Add small delay to avoid excessive CPU usage
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
}

/// UDP listener error types
#[derive(Debug)]
pub enum UdpListenerError {
    BindError(String, std::io::Error),
}

impl std::fmt::Display for UdpListenerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UdpListenerError::BindError(addr, err) => {
                write!(f, "Failed to bind to address {}: {}", addr, err)
            }
        }
    }
}

impl std::error::Error for UdpListenerError {}
