use anyhow::Result;
use log::{error, info};
use tokio::net::TcpListener;

use crate::config::{ProxyConfig, ProxyMode};
use crate::connection::ConnectionHandler;

pub struct TcpProxy {
    config: ProxyConfig,
}

impl TcpProxy {
    pub fn new(config: ProxyConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self) -> Result<()> {
        let mode_desc = self.get_mode_description();
        
        info!("Starting {mode_desc} on {}", self.config.listen_addr);
        
        let listener = TcpListener::bind(self.config.listen_addr).await?;
        info!("{mode_desc} listening on {}", self.config.listen_addr);
        
        loop {
            let (client_stream, client_addr) = listener.accept().await?;
            info!("New connection from {client_addr}");
            
            let mode = self.config.mode.clone();
            tokio::spawn(async move {
                let handler = ConnectionHandler::new(client_stream, client_addr, mode);
                if let Err(e) = handler.handle().await {
                    error!("Error handling client {client_addr}: {e}");
                }
            });
        }
    }
    
    fn get_mode_description(&self) -> String {
        match &self.config.mode {
            ProxyMode::Direct(addr) => format!("Direct proxy to {addr}"),
            ProxyMode::Socks4 => "SOCKS4 proxy".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProxyConfig;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_proxy_config_creation() {
        let config = ProxyConfig::new();
        assert_eq!(config.listen_addr, "127.0.0.1:1080".parse::<SocketAddr>().unwrap());
        assert!(matches!(config.mode, ProxyMode::Socks4));
    }

    #[tokio::test]
    async fn test_proxy_config_builder() {
        let listen_addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
        let target_addr = "127.0.0.1:9090".parse::<SocketAddr>().unwrap();
        
        let config = ProxyConfig::new()
            .with_listen_addr(listen_addr)
            .with_target_addr(target_addr);
            
        assert_eq!(config.listen_addr, listen_addr);
        assert!(matches!(config.mode, ProxyMode::Direct(_)));
    }
    
    #[test]
    fn test_mode_description() {
        let proxy = TcpProxy::new(ProxyConfig::new().with_socks4_mode());
        assert_eq!(proxy.get_mode_description(), "SOCKS4 proxy");
        
        let target = "127.0.0.1:8080".parse().unwrap();
        let proxy = TcpProxy::new(ProxyConfig::new().with_target_addr(target));
        assert_eq!(proxy.get_mode_description(), "Direct proxy to 127.0.0.1:8080");
    }
}