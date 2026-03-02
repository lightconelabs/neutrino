# Neutrino

Rust CLI for running SQL against a Trino coordinator.

## Build

```bash
cargo build --release
./target/release/neutrino --help
```

## Usage

```bash
neutrino --host trino.example.com "SELECT 1"
neutrino --host trino.example.com -f query.sql
neutrino --host trino.example.com --output json "SHOW CATALOGS"
```

## Auth Selection

- `--user` only: `X-Trino-User`
- `--user` + `--password`: Basic auth
- `--jwt-token` (optional `--user`): Bearer auth
- No auth flags: OAuth2 browser flow (token cached)

## Common Flags

- `--host` (required)
- `--port` (default `443`)
- `--catalog`
- `--schema`
- `--insecure` (skip TLS verification)
- `--limit`
- `--output table|json` (default `table`)
- `-f, --file` or positional `<QUERY>`

Env vars: `TRINO_HOST`, `TRINO_PORT`, `TRINO_USER`, `TRINO_CATALOG`, `TRINO_SCHEMA`, `TRINO_PASSWORD`, `TRINO_JWT_TOKEN`.

## Dev

```bash
cargo test
cargo run -- --host trino.example.com "SELECT 1"
```
