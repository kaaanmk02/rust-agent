# rust-agent

Rust-based agent gateway with HTTP/WebSocket server, multi-channel adapters, and CLI interface.

## Status

- Build: `cargo check` passes
- Tests: 2 passing
- Latest push: `master` branch on `https://github.com/kaaanmk02/rust-agent.git`

## Tech Stack

- Runtime: `tokio`
- HTTP/WS: `axum 0.7`
- CLI: `clap 4`
- Serialization: `serde` + `serde_json`
- HTTP client: `reqwest 0.12`
- Logging: `tracing` + `tracing-subscriber`
- CORS: `tower-http`
- Async traits: `async-trait`

## Project Summary

This project implements an agent gateway service in Rust. The gateway exposes HTTP and WebSocket endpoints for chat/tool orchestration, supports multiple channel adapters, and provides a CLI for runtime control.

Key modules:

- `src/main.rs` - Entry point with `clap` CLI: `serve`, `send`, `tools`
- `src/gateway/server.rs` - Axum HTTP server with routes: `/health`, `/ws`, `/api/chat`, `/api/tools`, `/channels`, `/api/gateway/config`; includes CORS, auth middleware, and WebSocket broadcast via `tokio::sync::broadcast`
- `src/gateway/gateway_types.rs` - Core types: `IncomingMessage`, `OutgoingMessage`, `ChannelEnvelope`, `DeliveryReceipt`, `GatewayClient`, and `Adapter` trait
- `src/gateway/gateway_config.rs` - Configuration structs for `GatewayConfig`, `TelegramConfig`, `DiscordConfig`, `SlackConfig`, `WhatsAppConfig`, `McpConfig`
- `src/gateway/adapters.rs` - Adapter implementations: `TelegramAdapter`, `DiscordAdapter`, `SlackAdapter`, `WhatsAppAdapter`
- `src/gateway/middleware.rs` - `auth_middleware` validating `Authorization: Bearer <token>` or `?token=` query against `GATEWAY_AUTH_TOKEN`
- `src/gateway/ws.rs` - WebSocket upgrade handler broadcasting messages to connected clients
- `src/gateway/cli.rs` - CLI command definitions

Runtime behavior:

- `serve` starts the gateway on `0.0.0.0:9090` with CORS enabled
- WebSocket clients receive broadcast messages through a `broadcast::Sender`
- Auth middleware is optional; when `GATEWAY_AUTH_TOKEN` is set, requests must provide matching credentials
- Channel adapters are currently scaffolded; Telegram adapter calls Telegram Bot API `sendMessage`
- Unit tests cover config defaults and overrides only

## CLI

```
cargo run -- serve --listen 0.0.0.0:9090
cargo run -- send --url http://localhost:9090 --channel telegram --to 123 --text "hello"
cargo run -- tools
```

## Environment

- `GATEWAY_AUTH_TOKEN` - optional bearer token for auth middleware

## Notes

- No API keys/tokens are stored in the repo; provide via env/config at runtime
- Adapter integrations are placeholders; `poll_updates` returns empty and `/api/chat` is a scaffold
