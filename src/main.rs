use anyhow::Result;
use clap::Parser;
use simple_tor::{ProxyConfig, TcpProxy};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "simple_tor")]
#[command(about = "A simple TCP proxy implementation in Rust")]
#[command(version = "0.1.0")]
struct Cli {
    #[arg(short, long, default_value = "127.0.0.1:1080")]
    #[arg(help = "Proxy listen address")]
    listen: SocketAddr,
    
    #[arg(short, long)]
    #[arg(help = "Target server address to forward connections to")]
    target: SocketAddr,
    
    #[arg(short, long, action = clap::ArgAction::Count)]
    #[arg(help = "Increase verbosity (-v for info, -vv for debug)")]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info", 
        2 => "debug",
        _ => "trace",
    };
    
    unsafe {
        std::env::set_var("RUST_LOG", log_level);
    }
    env_logger::init();
    
    let config = ProxyConfig::new()
        .with_listen_addr(cli.listen)
        .with_target_addr(cli.target);
    
    let proxy = TcpProxy::new(config);
    proxy.start().await
}