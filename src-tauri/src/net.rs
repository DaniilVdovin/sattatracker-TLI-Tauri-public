use reqwest::{Client, Response};
use std::time::Duration;

pub fn client(timeout: Duration) -> Result<Client, String> {
    Client::builder()
        .user_agent("TLI-Sattatracker")
        .https_only(true)
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())
}

pub async fn read_limited(mut resp: Response, max: usize) -> Result<Vec<u8>, String> {
    if let Some(len) = resp.content_length() {
        if len > max as u64 {
            return Err("response too large".to_string());
        }
    }
    let mut out = Vec::new();
    while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
        if out.len() + chunk.len() > max {
            return Err("response too large".to_string());
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}
