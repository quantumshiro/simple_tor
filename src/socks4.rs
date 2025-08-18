//! SOCKS4 protocol implementation
//!
//! This module implements the SOCKS4 protocol as defined in RFC 1928.
//! It supports CONNECT commands for establishing TCP connections through
//! the proxy server.
//!
//! # Protocol Overview
//!
//! SOCKS4 request format:
//! ```text
//! +----+----+----+----+----+----+----+----+----+----+....+----+
//! | VN | CD | DSTPORT |      DSTIP        | USERID       |NULL|
//! +----+----+----+----+----+----+----+----+----+----+....+----+
//!   1    1      2              4           variable       1
//! ```
//!
//! SOCKS4 response format:
//! ```text
//! +----+----+----+----+----+----+----+----+
//! | VN | CD | DSTPORT |      DSTIP        |
//! +----+----+----+----+----+----+----+----+
//!   1    1      2              4
//! ```

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use anyhow::{Result, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const SOCKS4_VERSION: u8 = 0x04;
const SOCKS4_CONNECT: u8 = 0x01;
const SOCKS4_BIND: u8 = 0x02;

const SOCKS4_REPLY_VERSION: u8 = 0x00;
const SOCKS4_REQUEST_GRANTED: u8 = 0x5A;
const SOCKS4_REQUEST_REJECTED: u8 = 0x5B;

#[derive(Debug, Clone)]
pub struct Socks4Request {
    pub command: u8,
    pub dst_port: u16,
    pub dst_ip: Ipv4Addr,
    pub userid: Vec<u8>,
}

impl Socks4Request {
    pub async fn read_from<R>(reader: &mut R) -> Result<Self>
    where
        R: AsyncReadExt + Unpin,
    {
        let version = reader.read_u8().await?;
        if version != SOCKS4_VERSION {
            bail!("Invalid SOCKS version: {:#x}, expected {:#x}", version, SOCKS4_VERSION);
        }
        
        let command = reader.read_u8().await?;
        if command != SOCKS4_CONNECT && command != SOCKS4_BIND {
            bail!("Unsupported SOCKS4 command: {:#x}", command);
        }
        
        let dst_port = reader.read_u16().await?;
        
        let mut ip_bytes = [0u8; 4];
        reader.read_exact(&mut ip_bytes).await?;
        let dst_ip = Ipv4Addr::from(ip_bytes);
        
        let mut userid = Vec::new();
        loop {
            let byte = reader.read_u8().await?;
            if byte == 0x00 {
                break;
            }
            userid.push(byte);
            if userid.len() > 256 {
                bail!("USERID too long");
            }
        }
        
        Ok(Socks4Request {
            command,
            dst_port,
            dst_ip,
            userid,
        })
    }
    
    pub fn destination_addr(&self) -> SocketAddrV4 {
        SocketAddrV4::new(self.dst_ip, self.dst_port)
    }
}

#[derive(Debug)]
pub struct Socks4Response {
    pub status: u8,
    pub dst_port: u16,
    pub dst_ip: Ipv4Addr,
}

impl Socks4Response {
    pub fn success(addr: SocketAddrV4) -> Self {
        Self {
            status: SOCKS4_REQUEST_GRANTED,
            dst_port: addr.port(),
            dst_ip: *addr.ip(),
        }
    }
    
    pub fn failure() -> Self {
        Self {
            status: SOCKS4_REQUEST_REJECTED,
            dst_port: 0,
            dst_ip: Ipv4Addr::new(0, 0, 0, 0),
        }
    }
    
    pub fn try_from_socket_addr(addr: SocketAddr) -> Result<Self> {
        match addr {
            SocketAddr::V4(addr_v4) => Ok(Self::success(addr_v4)),
            SocketAddr::V6(_) => {
                bail!("IPv6 addresses are not supported in SOCKS4");
            }
        }
    }
    
    pub async fn write_to<W>(&self, writer: &mut W) -> Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        writer.write_u8(SOCKS4_REPLY_VERSION).await?;
        writer.write_u8(self.status).await?;
        writer.write_u16(self.dst_port).await?;
        writer.write_all(&self.dst_ip.octets()).await?;
        writer.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[tokio::test]
    async fn test_socks4_request_parsing() {
        let request_bytes = vec![
            0x04, // version
            0x01, // connect command
            0x00, 0x50, // port 80
            0x42, 0x66, 0x07, 0x63, // IP 66.102.7.99
            0x46, 0x72, 0x65, 0x64, // userid "Fred"
            0x00, // null terminator
        ];
        
        let mut cursor = Cursor::new(request_bytes);
        let request = Socks4Request::read_from(&mut cursor).await.unwrap();
        
        assert_eq!(request.command, SOCKS4_CONNECT);
        assert_eq!(request.dst_port, 80);
        assert_eq!(request.dst_ip, Ipv4Addr::new(66, 102, 7, 99));
        assert_eq!(request.userid, b"Fred");
    }
    
    #[tokio::test]
    async fn test_socks4_request_invalid_version() {
        let request_bytes = vec![0x05, 0x01, 0x00, 0x50]; // SOCKS5 version
        
        let mut cursor = Cursor::new(request_bytes);
        let result = Socks4Request::read_from(&mut cursor).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid SOCKS version"));
    }
    
    #[tokio::test]
    async fn test_socks4_request_unsupported_command() {
        let request_bytes = vec![0x04, 0x03, 0x00, 0x50]; // unsupported command
        
        let mut cursor = Cursor::new(request_bytes);
        let result = Socks4Request::read_from(&mut cursor).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported SOCKS4 command"));
    }
    
    #[tokio::test]
    async fn test_socks4_response_generation() {
        let addr = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 1), 8080);
        let response = Socks4Response::success(addr);
        
        let mut buffer = Vec::new();
        response.write_to(&mut buffer).await.unwrap();
        
        assert_eq!(buffer[0], SOCKS4_REPLY_VERSION);
        assert_eq!(buffer[1], SOCKS4_REQUEST_GRANTED);
        assert_eq!(&buffer[2..4], &[0x1F, 0x90]); // port 8080 in big-endian
        assert_eq!(&buffer[4..8], &[192, 168, 1, 1]); // IP address
    }
    
    #[tokio::test]
    async fn test_socks4_response_failure() {
        let response = Socks4Response::failure();
        
        let mut buffer = Vec::new();
        response.write_to(&mut buffer).await.unwrap();
        
        assert_eq!(buffer[0], SOCKS4_REPLY_VERSION);
        assert_eq!(buffer[1], SOCKS4_REQUEST_REJECTED);
        assert_eq!(&buffer[2..4], &[0x00, 0x00]); // port 0
        assert_eq!(&buffer[4..8], &[0, 0, 0, 0]); // IP 0.0.0.0
    }
    
    #[test]
    fn test_socks4_response_from_socket_addr() {
        let addr_v4 = "192.168.1.1:8080".parse::<SocketAddr>().unwrap();
        let response = Socks4Response::try_from_socket_addr(addr_v4).unwrap();
        assert_eq!(response.status, SOCKS4_REQUEST_GRANTED);
        
        let addr_v6 = "[::1]:8080".parse::<SocketAddr>().unwrap();
        let result = Socks4Response::try_from_socket_addr(addr_v6);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("IPv6 addresses are not supported"));
    }
}