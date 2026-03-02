use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::time::{Duration, Instant};

use crate::auth::{Auth, AuthFlow};

const QUERY_POLL_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Debug, Deserialize)]
pub struct QueryResponse {
    pub id: Option<String>,
    #[serde(rename = "nextUri")]
    pub next_uri: Option<String>,
    pub columns: Option<Vec<Column>>,
    pub data: Option<Vec<Vec<serde_json::Value>>>,
    pub stats: Option<Stats>,
    pub error: Option<QueryError>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
}

#[derive(Debug, Deserialize)]
pub struct Stats {
    pub state: QueryState,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QueryState {
    Queued,
    Planning,
    Starting,
    Running,
    Finishing,
    Finished,
    Failed,
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for QueryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryState::Queued => write!(f, "QUEUED"),
            QueryState::Planning => write!(f, "PLANNING"),
            QueryState::Starting => write!(f, "STARTING"),
            QueryState::Running => write!(f, "RUNNING"),
            QueryState::Finishing => write!(f, "FINISHING"),
            QueryState::Finished => write!(f, "FINISHED"),
            QueryState::Failed => write!(f, "FAILED"),
            QueryState::Unknown(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryError {
    pub message: String,
    pub error_name: Option<String>,
    pub error_type: Option<String>,
}

pub struct TrinoClient {
    client: Client,
    base_url: String,
    auth: Auth,
    catalog: Option<String>,
    schema: Option<String>,
}

#[derive(Debug)]
pub struct QueryResult {
    columns: Vec<Column>,
    rows: Vec<Vec<serde_json::Value>>,
}

impl QueryResult {
    pub fn new(columns: Vec<Column>, rows: Vec<Vec<serde_json::Value>>) -> Result<Self> {
        let width = columns.len();
        for (i, row) in rows.iter().enumerate() {
            if row.len() != width {
                bail!(
                    "Row {} has {} values but there are {} columns",
                    i,
                    row.len(),
                    width
                );
            }
        }
        Ok(Self { columns, rows })
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn rows(&self) -> &[Vec<serde_json::Value>] {
        &self.rows
    }
}

impl TrinoClient {
    pub fn new(
        host: &str,
        port: u16,
        auth: AuthFlow,
        catalog: Option<String>,
        schema: Option<String>,
        insecure: bool,
    ) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(insecure)
            .build()
            .context("Failed to create HTTP client")?;
        let base_url = format!("https://{host}:{port}");
        let auth = auth.resolve(&client, &base_url)?;

        Ok(Self {
            client,
            base_url,
            auth,
            catalog,
            schema,
        })
    }

    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        self.auth.apply_headers(&mut headers)?;

        if let Some(ref catalog) = self.catalog {
            headers.insert(
                "X-Trino-Catalog",
                HeaderValue::from_str(catalog).context("Invalid X-Trino-Catalog header value")?,
            );
        }
        if let Some(ref schema) = self.schema {
            headers.insert(
                "X-Trino-Schema",
                HeaderValue::from_str(schema).context("Invalid X-Trino-Schema header value")?,
            );
        }

        Ok(headers)
    }

    pub fn execute(&self, sql: &str, limit: Option<usize>, quiet: bool) -> Result<QueryResult> {
        let resp = self
            .client
            .post(format!("{}/v1/statement", self.base_url))
            .headers(self.build_headers()?)
            .body(sql.to_string())
            .send()
            .context("Failed to submit query")?;

        if !resp.status().is_success() {
            bail!(
                "Query submission failed (HTTP {}): {}",
                resp.status(),
                resp.text().unwrap_or_default()
            );
        }

        let initial: QueryResponse = resp.json().context("Failed to parse query response")?;
        if let Some(ref error) = initial.error {
            bail!("Query error: {}", error.message);
        }

        self.poll_results(initial, limit, quiet)
    }

    fn poll_results(
        &self,
        initial: QueryResponse,
        limit: Option<usize>,
        quiet: bool,
    ) -> Result<QueryResult> {
        let mut columns = initial.columns.unwrap_or_default();
        let mut rows: Vec<Vec<serde_json::Value>> = initial.data.unwrap_or_default();
        let mut next_uri = initial.next_uri;
        let started = Instant::now();

        while let Some(uri) = next_uri {
            if started.elapsed() >= QUERY_POLL_TIMEOUT {
                bail!("Timed out waiting for query results after 600s");
            }

            if let Some(max) = limit
                && rows.len() >= max
            {
                rows.truncate(max);
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));

            let resp = self
                .client
                .get(&uri)
                .headers(self.build_headers()?)
                .send()
                .context("Failed to poll results")?;

            if !resp.status().is_success() {
                bail!(
                    "Polling failed (HTTP {}): {}",
                    resp.status(),
                    resp.text().unwrap_or_default()
                );
            }

            let page: QueryResponse = resp.json().context("Failed to parse poll response")?;
            if let Some(ref error) = page.error {
                bail!("Query error: {}", error.message);
            }

            if columns.is_empty() {
                columns = page.columns.unwrap_or_default();
            }
            if let Some(data) = page.data {
                rows.extend(data);
            }

            if !quiet {
                let state = page
                    .stats
                    .as_ref()
                    .map(|s| s.state.to_string())
                    .unwrap_or_else(|| "UNKNOWN".to_string());
                eprint!("\r\x1b[K{} ({} rows)", state, rows.len());
                let _ = std::io::Write::flush(&mut std::io::stderr());
            }

            next_uri = page.next_uri;
        }

        if let Some(max) = limit {
            rows.truncate(max);
        }

        if !quiet {
            eprint!("\r\x1b[K");
            let _ = std::io::Write::flush(&mut std::io::stderr());
        }

        QueryResult::new(columns, rows)
    }
}
