use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub listen_addr: SocketAddr,
    pub target_addr: SocketAddr,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:1080".parse().unwrap(),
            target_addr: "93.184.216.34:80".parse().unwrap(),
        }
    }
}

impl ProxyConfig {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_listen_addr(mut self, addr: SocketAddr) -> Self {
        self.listen_addr = addr;
        self
    }
    
    pub fn with_target_addr(mut self, addr: SocketAddr) -> Self {
        self.target_addr = addr;
        self
    }
}