use std::fmt;

#[derive(Debug)]
pub enum ProxyError {
    NetworkError(String),
    ConfigurationError(String),
    ConnectionClosed,
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            ProxyError::ConfigurationError(msg) => write!(f, "Configuration error: {msg}"),
            ProxyError::ConnectionClosed => write!(f, "Connection closed"),
        }
    }
}

impl std::error::Error for ProxyError {}

impl From<std::io::Error> for ProxyError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::UnexpectedEof => ProxyError::ConnectionClosed,
            _ => ProxyError::NetworkError(err.to_string()),
        }
    }
}