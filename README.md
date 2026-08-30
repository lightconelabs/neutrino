# Neutrino

A fast, single-binary Trino CLI. No JVM required.

Neutrino runs SQL against any Trino coordinator and returns results as formatted tables, JSON, or CSV. It ships as a single executable — no Java runtime, no Python environment, no dependencies to manage. Install it, point it at your cluster, and query.

## Install

**Homebrew** (macOS and Linux):

```bash
brew install lightconelabs/tap/neutrino-cli
```

**uv** (macOS, Linux, and Windows):

```bash
uv tool install neutrino-sql
```

This installs the `neutrino` command, and ships the same prebuilt binary as every other install method — Python is only used to deliver it, not to run it. `pipx install neutrino-sql` works the same way.

**Cargo** (from source):

```bash
cargo install --git https://github.com/lightconelabs/neutrino
```

**Binary download**: Grab a prebuilt binary from [GitHub Releases](https://github.com/lightconelabs/neutrino/releases). Prebuilt binaries cover macOS (Apple Silicon + Intel), Linux (ARM + x86), and Windows.

## Quick Start

Run an inline query:

```bash
neutrino --host trino.example.com "SELECT 1"
```

Run SQL from a file:

```bash
neutrino --host trino.example.com -f query.sql
```

Get JSON output for scripting:

```bash
neutrino --host trino.example.com --format json "SHOW CATALOGS"
```

Export results to CSV:

```bash
neutrino --host trino.example.com --format csv --output results.csv \
  "SELECT * FROM my_catalog.my_schema.my_table LIMIT 100"
```

## Authentication

Neutrino selects an auth method based on the flags you provide:

| Flags | Method |
|---|---|
| `--user` only | `X-Trino-User` header (no password) |
| `--user` + `--password` | HTTP Basic auth |
| `--jwt-token` (with optional `--user`) | Bearer token auth |
| No auth flags | OAuth2 browser flow (token cached for 1 hour) |

OAuth2 opens your browser, completes the flow, and caches the token locally. Subsequent queries reuse the cached token until it expires.

## Options

| Flag | Description | Default | Env var |
|---|---|---|---|
| `--host` | Trino coordinator hostname (required) | — | `TRINO_HOST` |
| `--port` | Coordinator port | `443` | `TRINO_PORT` |
| `--user` | Username | — | `TRINO_USER` |
| `--password` | Password (triggers Basic auth) | — | `TRINO_PASSWORD` |
| `--jwt-token` | JWT token (triggers Bearer auth) | — | `TRINO_JWT_TOKEN` |
| `--catalog` | Default catalog | — | `TRINO_CATALOG` |
| `--schema` | Default schema | — | `TRINO_SCHEMA` |
| `--insecure` | Skip TLS certificate verification | `false` | — |
| `--limit` | Maximum rows to return | — | — |
| `--format` | Output format: `table`, `json`, or `csv` | `table` | — |
| `-o, --output` | Write results to a file instead of stdout | — | — |
| `-f, --file` | Read SQL from a file | — | — |

You can set connection details once through environment variables and omit them from every command:

```bash
export TRINO_HOST=trino.example.com
export TRINO_CATALOG=my_catalog
neutrino "SELECT * FROM my_schema.my_table LIMIT 10"
```

## Output Formats

**Table** (default) — a formatted ASCII table with column names, types, and a row count:

```
┌──────────┬──────────┐
│ id       │ name     │
│ integer  │ varchar  │
├──────────┼──────────┤
│ 1        │ Alice    │
│ 2        │ Bob      │
└──────────┴──────────┘
(2 rows)
```

**JSON** — an array of objects, one per row. Useful for piping into `jq`:

```bash
neutrino --host trino.example.com --format json "SELECT id, name FROM users" | jq '.[].name'
```

**CSV** — RFC 4180 compliant, with a header row. Useful for spreadsheets and data pipelines:

```bash
neutrino --host trino.example.com --format csv "SELECT * FROM users" > users.csv
```

## Why Neutrino?

| | Neutrino | [trino-cli] | [trino-python-client] |
|---|---|---|---|
| Runtime dependency | None | [Java 11+][trino-cli-req] | [Python 3.9+][trino-python-req] |
| Startup time | Instant | Seconds ([JVM cold start][jvm-startup]) | Varies |
| OAuth2 browser flow | Built in, token cached | [Supported][trino-cli-oauth2] | [Supported][trino-python-oauth2] |
| Output formats | Table, JSON, CSV | [Multiple formats][trino-cli-output] | Custom code |
| Install | `brew install`, `uv tool install`, or single binary | [JAR download][trino-cli] + JVM | `pip install trino` |

[trino-cli]: https://trino.io/docs/current/client/cli.html
[trino-cli-req]: https://trino.io/docs/current/client/cli.html#requirements
[trino-cli-oauth2]: https://trino.io/docs/current/client/cli.html#external-authentication
[trino-cli-output]: https://trino.io/docs/current/client/cli.html#output-formats
[jvm-startup]: https://docs.oracle.com/en/java/javase/21/vm/class-data-sharing.html
[trino-python-client]: https://github.com/trinodb/trino-python-client
[trino-python-req]: https://github.com/trinodb/trino-python-client#requirements
[trino-python-oauth2]: https://github.com/trinodb/trino-python-client#oauth2-authentication

## Development

```bash
cargo test
cargo run -- --host trino.example.com "SELECT 1"
```

## License

[MIT](LICENSE)
