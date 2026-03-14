use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use reqwest::header::HeaderMap;

use crate::app_data::Data;

/// Fetches warpgate targets from the API and stores the result in `data`.
///
/// This sets `loading_targets` to `true` before fetching and `false` after,
/// and writes the result (or error) into `data.warpgate_targets`.
pub async fn fetch_warpgate_data(data: Data, config: Arc<Mutex<crate::config::AppConfig>>) {
    *data.loading_targets.lock().unwrap() = true;

    let warpgate_url = {
        let cfg = config.lock().unwrap();
        cfg.warpgate_api_url.clone()
    };

    let warpgate_token = {
        let cfg = config.lock().unwrap();
        cfg.warpgate_token.clone()
    };

    if warpgate_url.is_none() || warpgate_token.is_none() {
        *data.warpgate_targets.lock().unwrap() = Err(color_eyre::eyre::eyre!(
            "Warpgate API URL or token is not configured"
        ));
        *data.loading_targets.lock().unwrap() = false;
        return;
    }

    let result = fetch_targets(&warpgate_url.unwrap(), warpgate_token.as_deref()).await;

    match result {
        Ok(targets) => {
            *data.warpgate_targets.lock().unwrap() = Ok(targets);
        }
        Err(e) => {
            *data.warpgate_targets.lock().unwrap() = Err(e);
        }
    }

    *data.loading_targets.lock().unwrap() = false;
}

/// Performs the actual HTTP request to fetch warpgate targets.
async fn fetch_targets(
    url: &str,
    token: Option<&str>,
) -> color_eyre::Result<Vec<crate::warpgate::structs::WarpgateTarget>> {
    let mut headers = HeaderMap::new();
    if let Some(token) = token {
        headers.insert("X-Warpgate-Token", token.parse().unwrap());
    }
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .default_headers(headers)
        .timeout(Duration::from_millis(5_000))
        .build()?;

    // Get the list of warpgate targets from the endpoint
    let warpgate_targets: Vec<crate::warpgate::structs::WarpgateTarget> =
        client.get(url).send().await?.json().await?;

    Ok(warpgate_targets)
}
