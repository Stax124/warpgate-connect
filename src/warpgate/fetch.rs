use std::time::Duration;

use color_eyre::eyre::{Context, eyre};
use reqwest::header::{HeaderMap, HeaderValue};

use crate::config::AppConfig;
use crate::warpgate::target::WarpgateTarget;

pub async fn fetch_configured_targets(
    config: &AppConfig,
) -> color_eyre::Result<Vec<WarpgateTarget>> {
    let url = config
        .warpgate_api_url
        .as_deref()
        .ok_or_else(|| eyre!("Warpgate API URL is not configured"))?;
    let token = config
        .warpgate_token
        .as_deref()
        .ok_or_else(|| eyre!("Warpgate token is not configured"))?;

    tracing::info!(url = %url, "Fetching warpgate targets");
    fetch_targets(url, token).await
}

async fn fetch_targets(url: &str, token: &str) -> color_eyre::Result<Vec<WarpgateTarget>> {
    // Build the message here rather than forwarding the parse error, so nothing derived from the
    // token can reach a log line.
    let value: HeaderValue = token
        .parse()
        .map_err(|_| eyre!("Warpgate token is not a valid HTTP header value"))?;

    let mut headers = HeaderMap::new();
    headers.insert("X-Warpgate-Token", value);

    let client = reqwest::Client::builder()
        // Warpgate bastions commonly serve an internal or expired certificate; failing open keeps
        // the client usable, at the cost of not detecting an interception.
        .danger_accept_invalid_certs(true)
        .default_headers(headers)
        .timeout(Duration::from_millis(5_000))
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(eyre!("Warpgate API returned {status} for {url}"));
    }

    response
        .json()
        .await
        .with_context(|| format!("Could not decode the target list from {url}"))
}
