//! Where the API is.

use anyhow::{Context, Result, bail};
use reqwest::Url;

/// Overrides the API address, for a local server during development.
const BACKEND_URL_VAR: &str = "ZWIPE_BACKEND_URL";

/// The production API.
const DEFAULT_BACKEND_URL: &str = "https://api.zwipe.net";

/// The API address: `ZWIPE_BACKEND_URL` when set, else production.
///
/// # Errors
///
/// Fails when the address does not parse, or is plain `http` to anywhere but this machine, which would send the session token unencrypted.
pub(crate) fn backend_url() -> Result<Url> {
    parse_backend_url(std::env::var(BACKEND_URL_VAR).ok().as_deref())
}

fn parse_backend_url(value: Option<&str>) -> Result<Url> {
    let raw = value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(DEFAULT_BACKEND_URL);
    let url = Url::parse(raw).with_context(|| format!("{BACKEND_URL_VAR} is not a URL: {raw}"))?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if url.scheme() != "https" && !(url.scheme() == "http" && local) {
        bail!("{BACKEND_URL_VAR} must use https unless it points at this machine: {raw}");
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_is_the_default() {
        assert_eq!(
            parse_backend_url(None).unwrap().as_str(),
            "https://api.zwipe.net/"
        );
        assert_eq!(
            parse_backend_url(Some("  ")).unwrap().as_str(),
            "https://api.zwipe.net/"
        );
    }

    #[test]
    fn plain_http_is_allowed_only_to_this_machine() {
        assert!(parse_backend_url(Some("http://127.0.0.1:3000")).is_ok());
        assert!(parse_backend_url(Some("http://localhost:3000")).is_ok());
        assert!(parse_backend_url(Some("http://api.zwipe.net")).is_err());
        assert!(parse_backend_url(Some("ftp://127.0.0.1")).is_err());
        assert!(parse_backend_url(Some("not a url")).is_err());
    }
}
