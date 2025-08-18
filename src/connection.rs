//! Connection handling module
//!
//! This module handles individual client connections, including SOCKS4
//! handshake processing and bidirectional data forwarding.

use anyhow::Result;
use log::{error, info};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::config::ProxyMode;
use crate::socks4::{Socks4Request, Socks4Response};

/// Handles a single client connection through the proxy
pub struct ConnectionHandler {
    client_stream: TcpStream,
    client_addr: SocketAddr,
    mode: ProxyMode,
}

impl ConnectionHandler {
    pub fn new(client_stream: TcpStream, client_addr: SocketAddr, mode: ProxyMode) -> Self {
        Self {
            client_stream,
            client_addr,
            mode,
        }
    }

    pub async fn handle(mut self) -> Result<()> {
        let target_addr = match &self.mode {
            ProxyMode::Direct(addr) => *addr,
            ProxyMode::Socks4 => self.handle_socks4_handshake().await?,
        };

        info!("Connecting to target server: {target_addr}");

        let server_stream = match TcpStream::connect(target_addr).await {
            Ok(stream) => {
                info!("Connected to target server");
                stream
            }
            Err(e) => {
                error!("Failed to connect to target server: {e}");
                if matches!(self.mode, ProxyMode::Socks4) {
                    let response = Socks4Response::failure();
                    let _ = response.write_to(&mut self.client_stream).await;
                }
                return Err(e.into());
            }
        };

        if matches!(self.mode, ProxyMode::Socks4) {
            let response = Socks4Response::try_from_socket_addr(target_addr)
                .unwrap_or_else(|_| Socks4Response::failure());
            response.write_to(&mut self.client_stream).await?;
            info!("SOCKS4 handshake completed with {}", self.client_addr);
        }

        self.forward_traffic(server_stream).await
    }

    async fn handle_socks4_handshake(&mut self) -> Result<SocketAddr> {
        info!("Starting SOCKS4 handshake");

        let request = Socks4Request::read_from(&mut self.client_stream).await?;

        info!(
            "SOCKS4 request: command={:#x}, target={}:{}, userid={:?}",
            request.command,
            request.dst_ip,
            request.dst_port,
            String::from_utf8_lossy(&request.userid)
        );

        if request.command != 0x01 {
            let response = Socks4Response::failure();
            response.write_to(&mut self.client_stream).await?;
            anyhow::bail!("Unsupported SOCKS4 command: {:#x}", request.command);
        }

        Ok(SocketAddr::V4(request.destination_addr()))
    }

    async fn forward_traffic(self, server_stream: TcpStream) -> Result<()> {
        let (client_reader, client_writer) = self.client_stream.into_split();
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
}

pub async fn forward_data<R, W>(mut reader: R, mut writer: W, from: &str, to: &str) -> Result<()>
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