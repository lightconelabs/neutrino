pub mod auth;
pub mod client;
pub mod output;

use anyhow::{Context, Result, bail};
use auth::AuthFlow;
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ResolvedAuth {
    pub auth: AuthFlow,
    pub host: String,
    pub port: u16,
    pub catalog: Option<String>,
    pub schema: Option<String>,
    pub insecure: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlInput {
    File(PathBuf),
    Inline(String),
}

impl SqlInput {
    pub fn load(self) -> Result<String> {
        match self {
            SqlInput::File(path) => fs::read_to_string(&path)
                .with_context(|| format!("Failed to read SQL file: {}", path.display())),
            SqlInput::Inline(query) => Ok(query),
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "neutrino", version, about = "A Trino CLI built in Rust")]
pub struct Cli {
    /// Trino coordinator hostname
    #[arg(long, env = "TRINO_HOST")]
    pub host: String,

    /// Trino coordinator port
    #[arg(long, env = "TRINO_PORT", default_value = "443")]
    pub port: u16,

    /// Username for X-Trino-User header
    #[arg(long, env = "TRINO_USER")]
    pub user: Option<String>,

    /// Default catalog
    #[arg(long, env = "TRINO_CATALOG")]
    pub catalog: Option<String>,

    /// Default schema
    #[arg(long, env = "TRINO_SCHEMA")]
    pub schema: Option<String>,

    /// Password (implies BasicAuth)
    #[arg(long, env = "TRINO_PASSWORD")]
    pub password: Option<String>,

    /// JWT token (implies Bearer auth)
    #[arg(long, env = "TRINO_JWT_TOKEN")]
    pub jwt_token: Option<String>,

    /// Skip TLS certificate verification
    #[arg(long)]
    pub insecure: bool,

    /// Maximum number of rows to return
    #[arg(long)]
    pub limit: Option<usize>,

    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    pub output: OutputFormat,

    /// Read SQL from file
    #[arg(short, long, conflicts_with = "query")]
    pub file: Option<String>,

    /// SQL query (conflicts with --file)
    pub query: Option<String>,
}

impl Cli {
    pub fn resolve_sql_input(&self) -> Result<SqlInput> {
        match (&self.file, &self.query) {
            (Some(path), None) => Ok(SqlInput::File(PathBuf::from(path))),
            (None, Some(query)) => Ok(SqlInput::Inline(query.clone())),
            (None, None) => bail!("No query provided. Pass SQL as an argument or use -f <file>."),
            (Some(_), Some(_)) => bail!("Provide either --file or a SQL query, not both."),
        }
    }

    pub fn resolve_sql(&self) -> Result<String> {
        self.resolve_sql_input()?.load()
    }

    pub fn resolve_auth(self) -> Result<ResolvedAuth> {
        let auth = match (self.password, self.jwt_token, self.user) {
            (Some(password), _, Some(user)) => AuthFlow::Basic { user, password },
            (Some(_), _, None) => bail!("--user is required with --password"),
            (_, Some(token), user) => AuthFlow::Jwt { token, user },
            (_, _, Some(user)) => AuthFlow::None { user },
            _ => AuthFlow::OAuth2,
        };

        Ok(ResolvedAuth {
            auth,
            host: self.host,
            port: self.port,
            catalog: self.catalog,
            schema: self.schema,
            insecure: self.insecure,
        })
    }
}
