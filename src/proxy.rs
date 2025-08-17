use anyhow::Result;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::config::ProxyConfig;

pub struct TcpProxy {
    config: ProxyConfig,
}

impl TcpProxy {
    pub fn new(config: ProxyConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting TCP proxy on {}", self.config.listen_addr);
        
        let listener = TcpListener::bind(self.config.listen_addr).await?;
        info!("TCP proxy listening on {}", self.config.listen_addr);
        
        loop {
            let (client_stream, client_addr) = listener.accept().await?;
            info!("New connection from {client_addr}");
            
            let target_addr = self.config.target_addr;
            tokio::spawn(async move {
                if let Err(e) = handle_client_connection(client_stream, target_addr).await {
                    error!("Error handling client {client_addr}: {e}");
                }
            });
        }
    }
}

async fn handle_client_connection(client_stream: TcpStream, target_addr: std::net::SocketAddr) -> Result<()> {
    info!("Connecting to target server: {target_addr}");
    
    let server_stream = TcpStream::connect(target_addr).await?;
    info!("Connected to target server");
    
    let (client_reader, client_writer) = client_stream.into_split();
    let (server_reader, server_writer) = server_stream.into_split();
    
    let client_to_server = forward_data(client_reader, server_writer, "client", "server");
    let server_to_client = forward_data(server_reader, client_writer, "server", "client");
    
    tokio::select! {
        result = client_to_server => {
            if let Err(e) = result {
                error!("Error forwarding client to server: {e}");
            }
        }
        result = server_to_client => {
            if let Err(e) = result {
                error!("Error forwarding server to client: {e}");
            }
        }
    }
    
    info!("Connection closed");
    Ok(())
}

async fn forward_data<R, W>(mut reader: R, mut writer: W, from: &str, to: &str) -> Result<()>
where
    R: AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let mut buffer = vec![0u8; 4096];
    
    loop {
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        writer.write_all(&buffer[..n]).await?;
        writer.flush().await?;
        info!("Forwarded {n} bytes from {from} to {to}");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_proxy_config_creation() {
        let config = ProxyConfig::new();
        assert_eq!(config.listen_addr, "127.0.0.1:1080".parse::<SocketAddr>().unwrap());
        assert_eq!(config.target_addr, "93.184.216.34:80".parse::<SocketAddr>().unwrap());
    }

    #[tokio::test]
    async fn test_proxy_config_builder() {
        let listen_addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
        let target_addr = "127.0.0.1:9090".parse::<SocketAddr>().unwrap();
        
        let config = ProxyConfig::new()
            .with_listen_addr(listen_addr)
            .with_target_addr(target_addr);
            
        assert_eq!(config.listen_addr, listen_addr);
        assert_eq!(config.target_addr, target_addr);
    }

    #[tokio::test]
    async fn test_forward_data() {
        let test_data = b"Hello, World!";
        let mut reader = std::io::Cursor::new(test_data);
        let mut writer = Vec::new();
        
        forward_data(&mut reader, &mut writer, "test", "test")
            .await
            .unwrap();
            
        assert_eq!(writer, test_data);
    }
}