//! Simple Tor - A simplified Tor implementation for educational purposes
//!
//! This crate implements a basic SOCKS4 proxy server and demonstrates
//! the fundamental concepts of onion routing.
//!
//! # Architecture
//!
//! The implementation follows a phased approach:
//! - Phase 1: Basic TCP proxy
//! - Phase 2: SOCKS4 protocol implementation
//! - Phase 3: Multi-hop relay routing (planned)
//! - Phase 4: Encryption layer (planned)
//! - Phase 5: Onion routing (planned)
//!
//! # Examples
//!
//! ```rust,no_run
//! use simple_tor::{ProxyConfig, TcpProxy};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = ProxyConfig::new().with_socks4_mode();
//!     let proxy = TcpProxy::new(config);
//!     proxy.start().await
//! }
//! ```

pub mod config;
pub mod connection;
pub mod error;
pub mod proxy;
pub mod socks4;

pub use config::ProxyConfig;
pub use error::ProxyError;
pub use proxy::TcpProxy;