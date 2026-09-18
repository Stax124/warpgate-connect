use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use color_eyre::eyre::{Context, eyre};
use reqwest::header::{HeaderMap, HeaderValue};

use crate::app_data::Data;

/// Fetches warpgate targets from the API and stores the result, or the reason it failed, in
/// `data`.
pub async fn fetch_warpgate_data(data: Data, config: Arc<Mutex<crate::config::AppConfig>>) {
    *data.loading_targets.lock().unwrap() = true;

    let result = fetch_configured_targets(&config).await;
    match &result {
        Ok(targets) => tracing::info!(
            count = targets.len(),
            "Successfully fetched warpgate targets"
        ),
        Err(e) => tracing::error!(error = %e, "Failed to fetch warpgate targets"),
    }

    *data.warpgate_targets.lock().unwrap() = result;
    *data.loading_targets.lock().unwrap() = false;
}

pub(crate) async fn fetch_configured_targets(
    config: &Arc<Mutex<crate::config::AppConfig>>,
) -> color_eyre::Result<Vec<crate::warpgate::structs::WarpgateTarget>> {
    let (warpgate_url, warpgate_token) = {
        let cfg = config.lock().unwrap();
        (cfg.warpgate_api_url.clone(), cfg.warpgate_token.clone())
    };

    let url = warpgate_url.ok_or_else(|| eyre!("Warpgate API URL is not configured"))?;
    let token = warpgate_token.ok_or_else(|| eyre!("Warpgate token is not configured"))?;

    tracing::info!(url = %url, "Fetching warpgate targets");
    fetch_targets(&url, &token).await
}

async fn fetch_targets(
    url: &str,
    token: &str,
) -> color_eyre::Result<Vec<crate::warpgate::structs::WarpgateTarget>> {
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
