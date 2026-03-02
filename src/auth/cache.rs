use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(serde::Serialize, serde::Deserialize)]
struct CachedToken {
    token: String,
    expiration: u64,
}

fn path(base_url: &str) -> Result<PathBuf> {
    let dir = dirs::cache_dir()
        .context("Could not determine cache directory")?
        .join("neutrino");
    fs::create_dir_all(&dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))
            .context("Failed to set permissions on OAuth2 cache directory")?;
    }
    Ok(dir.join(format!(
        "oauth2_{:.16}.json",
        format!("{:x}", Sha256::digest(base_url.as_bytes()))
    )))
}

pub fn load(base_url: &str) -> Result<Option<String>> {
    let path = path(base_url)?;
    if !path.exists() {
        return Ok(None);
    }

    let cached: CachedToken = serde_json::from_str(&fs::read_to_string(&path)?)?;

    if SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() < cached.expiration {
        Ok(Some(cached.token))
    } else {
        fs::remove_file(&path)?;
        Ok(None)
    }
}

pub fn save(base_url: &str, token: &str, expiration: u64) -> Result<()> {
    let cache_path = path(base_url)?;
    let payload = serde_json::to_string_pretty(&CachedToken {
        token: token.to_string(),
        expiration,
    })?;

    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&cache_path)
            .context("Failed to open OAuth2 cache file")?;
        file.write_all(payload.as_bytes())
            .context("Failed to write OAuth2 cache file")?;
        file.flush().context("Failed to flush OAuth2 cache file")?;
    }

    #[cfg(not(unix))]
    {
        fs::write(&cache_path, payload).context("Failed to write OAuth2 cache file")?;
    }

    Ok(())
}
