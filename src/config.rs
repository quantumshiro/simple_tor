//! Configuration module for the Simple Tor proxy
//!
//! This module defines the configuration structures and proxy modes
//! supported by the Simple Tor implementation.

use std::net::SocketAddr;

/// Defines the operating mode of the proxy server
#[derive(Debug, Clone)]
pub enum ProxyMode {
    /// Direct TCP proxy to a fixed target address
    Direct(SocketAddr),
    /// SOCKS4 proxy allowing dynamic target selection
    Socks4,
}

/// Configuration for the proxy server
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Address to bind the proxy server to
    pub listen_addr: SocketAddr,
    /// Operating mode of the proxy
    pub mode: ProxyMode,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:1080".parse().unwrap(),
            mode: ProxyMode::Socks4,
        }
    }
}

impl ProxyConfig {
    /// Create a new proxy configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the listen address for the proxy server
    pub fn with_listen_addr(mut self, addr: SocketAddr) -> Self {
        self.listen_addr = addr;
        self
    }
    
    /// Configure as a direct proxy to a specific target
    pub fn with_target_addr(mut self, addr: SocketAddr) -> Self {
        self.mode = ProxyMode::Direct(addr);
        self
    }
    
    /// Configure as a SOCKS4 proxy
    pub fn with_socks4_mode(mut self) -> Self {
        self.mode = ProxyMode::Socks4;
        self
    }
    
    /// Set buffer size for data forwarding (default: 4096)
    pub fn with_buffer_size(self, _size: usize) -> Self {
        // For future use when buffer size becomes configurable
        self
    }
    
    /// Set connection timeout in seconds (default: 30)
    pub fn with_timeout(self, _seconds: u64) -> Self {
        // For future use when timeouts become configurable
        self
    }
}