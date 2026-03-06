mod download;

pub use download::download_mod_file;

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;
use ts_rs::TS;
use url::Url;

/// Parsed representation of a `ltk://install` deep-link URL.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DeepLinkInstallRequest {
    pub url: String,
    pub name: Option<String>,
    pub author: Option<String>,
    pub source: Option<String>,
}

/// Progress payload emitted during protocol install download.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolInstallProgress {
    pub stage: String,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub error: Option<String>,
}

/// Rate-limiter state for deep-link invocations.
pub struct DeepLinkState {
    last_invocation: Mutex<Option<Instant>>,
}

impl DeepLinkState {
    pub fn new() -> Self {
        Self {
            last_invocation: Mutex::new(None),
        }
    }

    /// Returns `true` if the invocation should be dropped (rate-limited).
    pub fn should_rate_limit(&self) -> bool {
        let mut last = self.last_invocation.lock().unwrap();
        let now = Instant::now();
        if let Some(prev) = *last {
            if now.duration_since(prev).as_secs_f64() < 1.0 {
                return true;
            }
        }
        *last = Some(now);
        false
    }
}

/// Parse and validate a `ltk://install?...` deep-link URL.
pub fn parse_deep_link_url(raw_url: &str) -> AppResult<DeepLinkInstallRequest> {
    let parsed = Url::parse(raw_url)
        .map_err(|e| AppError::ValidationFailed(format!("Malformed deep-link URL: {e}")))?;

    if parsed.scheme() != "ltk" {
        return Err(AppError::ValidationFailed(format!(
            "Expected 'ltk' scheme, got '{}'",
            parsed.scheme()
        )));
    }

    // The host portion of ltk://install is "install"
    let host = parsed.host_str().unwrap_or("");
    let path = parsed.path().trim_start_matches('/');
    let action = if !host.is_empty() { host } else { path };
    if action != "install" {
        return Err(AppError::ValidationFailed(format!(
            "Unknown action '{action}', only 'install' is supported"
        )));
    }

    let pairs: std::collections::HashMap<String, String> =
        parsed.query_pairs().into_owned().collect();

    let download_url = pairs
        .get("url")
        .ok_or_else(|| AppError::ValidationFailed("Missing required 'url' parameter".into()))?;

    let download_parsed = Url::parse(download_url)
        .map_err(|e| AppError::ValidationFailed(format!("Invalid download URL: {e}")))?;

    if download_parsed.scheme() != "https" {
        return Err(AppError::ValidationFailed(
            "Download URL must use HTTPS".into(),
        ));
    }

    // SSRF prevention: reject loopback/private hosts
    if let Some(host) = download_parsed.host_str() {
        let lower = host.to_lowercase();
        if lower == "localhost"
            || lower == "127.0.0.1"
            || lower == "::1"
            || lower.starts_with("10.")
            || lower.starts_with("192.168.")
            || lower == "0.0.0.0"
        {
            return Err(AppError::ValidationFailed(
                "Download URL must not point to a local/private address".into(),
            ));
        }
        // Check 172.16.0.0/12 range
        if lower.starts_with("172.") {
            if let Some(second_octet) = lower
                .strip_prefix("172.")
                .and_then(|s| s.split('.').next())
                .and_then(|s| s.parse::<u8>().ok())
            {
                if (16..=31).contains(&second_octet) {
                    return Err(AppError::ValidationFailed(
                        "Download URL must not point to a local/private address".into(),
                    ));
                }
            }
        }
    }

    let name = pairs.get("name").map(|s| truncate_str(s, 256).to_string());
    let author = pairs
        .get("author")
        .map(|s| truncate_str(s, 256).to_string());
    let source = pairs
        .get("source")
        .map(|s| truncate_str(s, 256).to_string());

    Ok(DeepLinkInstallRequest {
        url: download_url.clone(),
        name,
        author,
        source,
    })
}

/// Emit a completion progress event.
pub fn emit_install_complete(app_handle: &tauri::AppHandle) {
    use tauri::Emitter;
    let _ = app_handle.emit(
        "protocol-install-progress",
        ProtocolInstallProgress {
            stage: "complete".to_string(),
            bytes_downloaded: 0,
            total_bytes: None,
            error: None,
        },
    );
}

fn truncate_str(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_minimal_url() {
        let req = parse_deep_link_url("ltk://install?url=https://cdn.example.com/mods/skin.modpkg")
            .unwrap();
        assert_eq!(req.url, "https://cdn.example.com/mods/skin.modpkg");
        assert!(req.name.is_none());
        assert!(req.source.is_none());
    }

    #[test]
    fn parse_valid_full_url() {
        let req = parse_deep_link_url(
            "ltk://install?url=https://cdn.example.com/mods/skin.modpkg&name=Cool%20Skin&author=SkinMaker&source=MySite",
        )
        .unwrap();
        assert_eq!(req.name.as_deref(), Some("Cool Skin"));
        assert_eq!(req.author.as_deref(), Some("SkinMaker"));
        assert_eq!(req.source.as_deref(), Some("MySite"));
    }

    #[test]
    fn rejects_http_url() {
        let err = parse_deep_link_url("ltk://install?url=http://cdn.example.com/mods/skin.modpkg");
        assert!(err.is_err());
    }

    #[test]
    fn rejects_missing_url_param() {
        let err = parse_deep_link_url("ltk://install?name=Test");
        assert!(err.is_err());
    }

    #[test]
    fn rejects_localhost() {
        let err = parse_deep_link_url("ltk://install?url=https://localhost/mods/skin.modpkg");
        assert!(err.is_err());
    }

    #[test]
    fn rejects_private_ip() {
        let err = parse_deep_link_url("ltk://install?url=https://192.168.1.1/mods/skin.modpkg");
        assert!(err.is_err());
    }

    #[test]
    fn ignores_unknown_params() {
        let req = parse_deep_link_url(
            "ltk://install?url=https://cdn.example.com/mods/skin.modpkg&checksum=sha256:abc&api=https://api.example.com&unknown=value",
        )
        .unwrap();
        assert_eq!(req.url, "https://cdn.example.com/mods/skin.modpkg");
    }

    #[test]
    fn rejects_wrong_scheme() {
        let err =
            parse_deep_link_url("https://install?url=https://cdn.example.com/mods/skin.modpkg");
        assert!(err.is_err());
    }

    #[test]
    fn rejects_unknown_action() {
        let err = parse_deep_link_url("ltk://update?url=https://cdn.example.com/mods/skin.modpkg");
        assert!(err.is_err());
    }

    #[test]
    fn rate_limiter_blocks_rapid_calls() {
        let state = DeepLinkState::new();
        assert!(!state.should_rate_limit());
        assert!(state.should_rate_limit());
    }
}
