use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use reqwest::header::WWW_AUTHENTICATE;
use std::time::{Duration, Instant};

const TOKEN_POLL_TIMEOUT: Duration = Duration::from_secs(120);

pub fn authenticate(client: &Client, base_url: &str) -> Result<String> {
    let resp = client
        .post(format!("{base_url}/v1/statement"))
        .body("SELECT 1")
        .send()
        .context("Failed to initiate OAuth2 flow")?;

    if resp.status() != reqwest::StatusCode::UNAUTHORIZED {
        bail!("Expected 401 for OAuth2 flow, got {}", resp.status());
    }

    let www_auth = resp
        .headers()
        .get_all(WWW_AUTHENTICATE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.contains("x_redirect_server"))
        .context("No WWW-Authenticate header with x_redirect_server found")?
        .to_string();

    let redirect_server = parse_auth_param(&www_auth, "x_redirect_server")
        .context("Missing x_redirect_server in WWW-Authenticate")?;
    let token_server = parse_auth_param(&www_auth, "x_token_server")
        .context("Missing x_token_server in WWW-Authenticate")?;

    eprintln!("Opening browser for OAuth2 authentication...");
    open::that(&redirect_server).context("Failed to open browser")?;

    eprintln!("Waiting for authentication...");
    poll_token_server(client, &token_server)
}

fn parse_auth_param(header: &str, param: &str) -> Option<String> {
    let search = format!("{param}=\"");
    let start = header.find(&search)? + search.len();
    let end = start + header[start..].find('"')?;
    Some(header[start..end].to_string())
}

fn poll_token_server(client: &Client, token_server: &str) -> Result<String> {
    let mut url = token_server.to_string();
    let started = Instant::now();

    loop {
        if started.elapsed() >= TOKEN_POLL_TIMEOUT {
            bail!("Timed out waiting for OAuth2 token after 120s");
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        let resp = client
            .get(&url)
            .send()
            .context("Failed to poll token server")?;

        if resp.status() != reqwest::StatusCode::OK {
            continue;
        }

        let body = resp.text()?;

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            if let Some(next) = json.get("nextUri").and_then(|u| u.as_str()) {
                url = next.to_string();
                continue;
            }
            if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                return Ok(token.to_string());
            }
        }

        let trimmed = body.trim();
        if looks_like_token(trimmed) {
            return Ok(trimmed.to_string());
        }
    }
}

fn looks_like_token(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~' | '/' | '+' | '=')
        })
}
