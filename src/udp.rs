use tokio::net::UdpSocket;
use std::time::Duration;

/// UDP监听器，用于接收来自Ham Radio软件的ADIF数据
#[derive(Debug)]
pub struct UdpListener {
    host: String,
    port: u16,
}

impl UdpListener {
    /// 创建新的UDP监听器实例
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    /// 启动UDP服务器并监听数据
    /// 返回接收到的第一个数据包的内容
    pub async fn listen_once(&self) -> Result<Vec<u8>, UdpListenerError> {
        let addr = format!("{}:{}", self.host, self.port);
        
        let sock = UdpSocket::bind(&addr).await
            .map_err(|e| UdpListenerError::BindError(addr.clone(), e))?;

        let mut buf = [0; 1024];
        
        loop {
            match sock.recv_from(&mut buf).await {
                Ok((len, _src)) => {
                    return Ok(buf[..len].to_vec());
                }
                Err(e) => {
                    // 记录错误但继续监听
                    eprintln!("UDP receive error: {}", e);
                    // 添加小延迟避免过度占用CPU
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        }
    }
}

/// UDP监听器错误类型
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
