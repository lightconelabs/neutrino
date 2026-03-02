mod cache;
mod oauth2;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub enum Auth {
    None { user: String },
    Basic { user: String, password: String },
    Jwt { token: String, user: Option<String> },
}

#[derive(Debug, Clone)]
pub enum AuthFlow {
    None { user: String },
    Basic { user: String, password: String },
    Jwt { token: String, user: Option<String> },
    OAuth2,
}

impl Auth {
    pub fn apply_headers(&self, headers: &mut HeaderMap) -> Result<()> {
        match self {
            Auth::None { user } => {
                headers.insert(
                    "X-Trino-User",
                    HeaderValue::from_str(user).context("Invalid X-Trino-User header value")?,
                );
            }
            Auth::Basic { user, password } => {
                headers.insert(
                    "X-Trino-User",
                    HeaderValue::from_str(user).context("Invalid X-Trino-User header value")?,
                );
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!(
                        "Basic {}",
                        BASE64.encode(format!("{user}:{password}"))
                    ))
                    .context("Invalid Authorization header value")?,
                );
            }
            Auth::Jwt { token, user } => {
                if let Some(u) = user {
                    headers.insert(
                        "X-Trino-User",
                        HeaderValue::from_str(u).context("Invalid X-Trino-User header value")?,
                    );
                }
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {token}"))
                        .context("Invalid Authorization header value")?,
                );
            }
        }
        Ok(())
    }
}

impl AuthFlow {
    pub fn resolve(self, client: &Client, base_url: &str) -> Result<Auth> {
        match self {
            AuthFlow::None { user } => Ok(Auth::None { user }),
            AuthFlow::Basic { user, password } => Ok(Auth::Basic { user, password }),
            AuthFlow::Jwt { token, user } => Ok(Auth::Jwt { token, user }),
            AuthFlow::OAuth2 => self.resolve_oauth2(client, base_url),
        }
    }

    fn resolve_oauth2(self, client: &Client, base_url: &str) -> Result<Auth> {
        if let Some(cached) = cache::load(base_url)? {
            return Ok(Auth::Jwt {
                token: cached,
                user: None,
            });
        }

        let token = oauth2::authenticate(client, base_url)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        cache::save(base_url, &token, now + 3600)?;

        Ok(Auth::Jwt { token, user: None })
    }
}
