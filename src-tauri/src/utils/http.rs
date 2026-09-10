use reqwest::{Client, Response};
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::utils::APP_USER_AGENT;

pub fn http_client() -> AppResult<Client> {
    Client::builder()
        .user_agent(APP_USER_AGENT)
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(300))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| AppError::message(e.to_string()))
}

pub async fn get_json<T: serde::de::DeserializeOwned>(client: &Client, url: &str) -> AppResult<T> {
    let response = send(client, url).await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(AppError::new(
            "Provider API unavailable",
            "The selected server platform could not be queried.",
        )
        .with_causes(vec![
            "The provider metadata service may be down",
            "The selected version may no longer exist",
            "Internet connection unavailable",
        ])
        .with_technical(format!("{url} -> {status}: {text}")));
    }
    serde_json::from_str(&text).map_err(|e| {
        AppError::new(
            "Provider API unavailable",
            "The provider returned data ServerForge could not understand.",
        )
        .with_technical(format!("{url}: {e}; body={text}"))
    })
}

pub async fn send(client: &Client, url: &str) -> AppResult<Response> {
    client.get(url).send().await.map_err(Into::into)
}

pub async fn get_text(client: &Client, url: &str) -> AppResult<String> {
    let response = send(client, url).await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(AppError::new(
            "Provider API unavailable",
            "Unable to download metadata from the provider.",
        )
        .with_technical(format!("{url} -> {status}: {text}")));
    }
    Ok(text)
}
