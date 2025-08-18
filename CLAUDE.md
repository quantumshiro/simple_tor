# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a simplified Tor implementation in Rust that demonstrates the fundamentals of onion routing. The project is designed to be built in phases, starting from a basic TCP proxy and gradually adding features like SOCKS4 protocol support, relay nodes, encryption, and onion routing.

## Common Development Commands

```bash
# Build the project
cargo build

# Run in development mode
cargo run

# Build for release
cargo build --release

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Architecture Plan

The implementation follows a phased approach:

1. **Phase 1**: Basic TCP proxy (passthrough) - Simple data forwarding between client and target
2. **Phase 2**: SOCKS4 protocol implementation - Handle SOCKS4 CONNECT commands
3. **Phase 3**: Relay node implementation - Multi-hop routing through entry, relay, and exit nodes
4. **Phase 4**: Encryption layer - AES encryption and Diffie-Hellman key exchange
5. **Phase 5**: Onion routing - Multi-layered encryption and circuit-based data transfer

## SOCKS4 Protocol Details

The project uses SOCKS4 instead of SOCKS5 for simplicity:
- Request format: `[VER(0x04)][CMD(0x01)][DSTPORT(2bytes)][DSTIP(4bytes)][USERID][NULL]`
- Response format: `[0x00][STATUS][PORT(2bytes)][IP(4bytes)]`
- Default listening port: 1080

## Development Notes

- The project uses async Rust with tokio for handling concurrent connections
- Each phase builds upon the previous one, maintaining backward compatibility
- Focus on educational clarity over production-level security