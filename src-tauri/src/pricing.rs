use lazy_static::lazy_static;
use reqwest::Client;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

lazy_static! {
    static ref PRICE_CACHE: Mutex<HashMap<String, f64>> = Mutex::new(HashMap::new());
    static ref BLOCKED_ITEMS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

static FULL_PRICE_LOADED: AtomicBool = AtomicBool::new(false);

pub fn get_all_cached_prices() -> Vec<(String, f64)> {
    let cache = PRICE_CACHE.lock().unwrap();
    cache.iter().map(|(k, &v)| (k.clone(), v)).collect()
}

pub fn clear_price_cache() {
    let mut cache = PRICE_CACHE.lock().unwrap();
    cache.clear();
    FULL_PRICE_LOADED.store(false, Ordering::Relaxed);
    println!("[pricing] Price cache cleared");
}

pub struct PricingManager {
    client: Client,
    season_id: u32,
    cache_path: PathBuf,
}

impl PricingManager {
    pub fn new(season_id: u32, cache_path: PathBuf) -> Self {
        Self {
            client: crate::net::client(std::time::Duration::from_secs(10)).unwrap_or_default(),
            season_id,
            cache_path,
        }
    }

    pub async fn load_disk_cache(&self) {
        match tokio::fs::metadata(&self.cache_path).await {
            Ok(meta) if meta.len() as usize <= MAX_TABLE_BYTES => {}
            _ => return,
        }
        let data = match tokio::fs::read_to_string(&self.cache_path).await {
            Ok(d) => d,
            Err(_) => return,
        };
        let raw: HashMap<String, serde_json::Value> = match serde_json::from_str(&data) {
            Ok(m) => m,
            Err(_) => return,
        };
        let map: HashMap<String, f64> = raw
            .into_iter()
            .filter_map(|(id, v)| {
                match (valid_item_id(&id), v.as_f64().filter(|p| valid_price(*p))) {
                    (true, Some(price)) => Some((id, price)),
                    _ => None,
                }
            })
            .take(MAX_TABLE_ENTRIES)
            .collect();
        let count = map.len();
        {
            let mut cache = PRICE_CACHE.lock().unwrap();
            for (k, v) in map {
                cache.insert(k, v);
            }
        }
        FULL_PRICE_LOADED.store(true, Ordering::Relaxed);
        println!("[pricing] Loaded {} prices from disk cache", count);
    }

    async fn save_disk_cache(&self) {
        let data = {
            let cache = PRICE_CACHE.lock().unwrap();
            match serde_json::to_string(&*cache) {
                Ok(s) => s,
                Err(_) => return,
            }
        };
        if let Err(e) = tokio::fs::write(&self.cache_path, data).await {
            eprintln!("[pricing] Failed to save price cache to disk: {}", e);
        }
    }

    pub async fn get_price(&self, item_id: &str) -> f64 {
        if item_id == "100300" {
            return 1.0;
        }
        if item_id.ends_with("300") && item_id.len() > 3 {
            return 0.0;
        }

        {
            let blocked = BLOCKED_ITEMS.lock().unwrap();
            if blocked.contains(item_id) {
                return 0.0;
            }
        }

        {
            let cache = PRICE_CACHE.lock().unwrap();
            if let Some(&price) = cache.get(item_id) {
                return price;
            }
        }

        if FULL_PRICE_LOADED.load(Ordering::Relaxed) {
            let mut cache = PRICE_CACHE.lock().unwrap();
            cache.insert(item_id.to_string(), 0.0);
            return 0.0;
        }

        let price = self.fetch_price_from_api(item_id).await;

        let mut cache = PRICE_CACHE.lock().unwrap();
        cache.insert(item_id.to_string(), price);

        price
    }

    pub fn get_price_sync(&self, item_id: &str) -> f64 {
        if item_id == "100300" {
            return 1.0;
        }
        if item_id.ends_with("300") && item_id.len() > 3 {
            return 0.0;
        }
        {
            let blocked = BLOCKED_ITEMS.lock().unwrap();
            if blocked.contains(item_id) {
                return 0.0;
            }
        }
        let cache = PRICE_CACHE.lock().unwrap();
        cache.get(item_id).copied().unwrap_or(0.0)
    }

    pub fn update_local_price(item_id: &str, price: f64) {
        let mut cache = PRICE_CACHE.lock().unwrap();
        cache.insert(item_id.to_string(), price);
        println!(
            "[pricing] Auction price for {} updated to {} FE",
            item_id, price
        );
    }

    async fn download_table(&self) -> Result<HashMap<String, f64>, String> {
        let mut season = self.season_id.to_string();
        for attempt in 0..2 {
            match self
                .get_json(&format!("/api/prices-snapshot/{}", season))
                .await?
            {
                Some(body) => {
                    let table = parse_snapshot(&body);
                    if !table.is_empty() {
                        return Ok(table);
                    }
                }
                None => {}
            }
            if attempt == 1 {
                break;
            }
            let status = self.get_json("/api/player/season-status").await?;
            let current = status
                .and_then(|s| {
                    s.get("currentSeasonId")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                })
                .ok_or_else(|| "season-status has no currentSeasonId".to_string())?;
            if current == season {
                break;
            }
            println!(
                "[pricing] No snapshot for season {}, using current season {}",
                season, current
            );
            season = current;
        }
        Err(format!("no price data for season {}", season))
    }

    async fn get_json(&self, path: &str) -> Result<Option<Value>, String> {
        let mut last_err = String::new();
        for base in etor_bases() {
            let url = format!("{}{}", base, path);
            match self
                .client
                .get(&url)
                .header("User-Agent", "TLI-Sattatracker")
                .send()
                .await
            {
                Ok(resp) if resp.status().as_u16() == 400 || resp.status().as_u16() == 404 => {
                    return Ok(None);
                }
                Ok(resp) if resp.status().is_success() => {
                    match crate::net::read_limited(resp, MAX_TABLE_BYTES).await {
                        Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                            Ok(json) => return Ok(Some(json)),
                            Err(e) => last_err = format!("{}: {}", url, e),
                        },
                        Err(e) => last_err = format!("{}: {}", url, e),
                    }
                }
                Ok(resp) => last_err = format!("{}: HTTP {}", url, resp.status()),
                Err(e) => last_err = format!("{}: {}", url, e),
            }
        }
        Err(last_err)
    }

    pub async fn refresh_prices(&self) {
        match self.download_table().await {
            Ok(table) => {
                let count = table.len();
                {
                    let mut cache = PRICE_CACHE.lock().unwrap();
                    *cache = table;
                }
                FULL_PRICE_LOADED.store(true, Ordering::Relaxed);
                self.save_disk_cache().await;
                println!(
                    "[pricing] Price table refreshed from ETor ({} items) and saved to disk",
                    count
                );
            }
            Err(e) => {
                eprintln!("[pricing] ETor error (keeping last known prices): {}", e);
                FULL_PRICE_LOADED.store(true, Ordering::Relaxed);
            }
        }
    }

    async fn fetch_price_from_api(&self, item_id: &str) -> f64 {
        self.refresh_prices().await;
        let cache = PRICE_CACHE.lock().unwrap();
        cache.get(item_id).copied().unwrap_or(0.0)
    }
}

const MAX_TABLE_BYTES: usize = 8 * 1024 * 1024;
const MAX_TABLE_ENTRIES: usize = 20_000;
const MAX_PRICE: f64 = 10_000_000.0;

fn valid_item_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 12 && id.bytes().all(|b| b.is_ascii_digit())
}

fn valid_price(price: f64) -> bool {
    price.is_finite() && (0.0..=MAX_PRICE).contains(&price)
}

fn parse_snapshot(body: &Value) -> HashMap<String, f64> {
    let mut table = HashMap::new();
    let rows = match body.get("data").and_then(|d| d.as_array()) {
        Some(rows) => rows,
        None => return table,
    };
    for row in rows.iter().take(MAX_TABLE_ENTRIES) {
        let id = match row.get("id").and_then(|v| v.as_str()) {
            Some(id) if valid_item_id(id) => id,
            _ => continue,
        };
        let price = row.get("price").and_then(|p| {
            p.as_f64()
                .or_else(|| p.as_str().and_then(|s| s.parse::<f64>().ok()))
        });
        if let Some(price) = price.filter(|p| valid_price(*p)) {
            table.insert(id.to_string(), price);
        }
    }
    table
}

const ETOR_PRIMARY: &str = "https://etor-zero.981001.xyz/etor-api";
const ETOR_FALLBACK: &str = "https://etor-beta.710421059.xyz/etor-api";

fn etor_bases() -> Vec<String> {
    let mut bases = Vec::new();
    if let Ok(custom) = std::env::var("TLI_PRICE_API_URL") {
        let custom = custom.trim();
        if custom.starts_with("https://") {
            bases.push(custom.trim_end_matches('/').to_string());
        }
    }
    bases.push(ETOR_PRIMARY.to_string());
    bases.push(ETOR_FALLBACK.to_string());
    bases
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn valid_rows_are_parsed() {
        let body = json!({"data": [
            {"id": "350513", "price": 910.8},
            {"id": "7744", "price": "8.006"},
            {"id": "1", "price": 0}
        ]});
        let t = parse_snapshot(&body);
        assert_eq!(t.len(), 3);
        assert_eq!(t["350513"], 910.8);
        assert_eq!(t["7744"], 8.006);
        assert_eq!(t["1"], 0.0);
    }

    #[test]
    fn hostile_rows_are_dropped() {
        let body = json!({"data": [
            {"id": "../etc", "price": 1},
            {"id": "12345678901234", "price": 1},
            {"id": "", "price": 1},
            {"id": "5", "price": -1},
            {"id": "6", "price": 1e30},
            {"id": "7", "price": "NaN"},
            {"id": "8", "price": "inf"},
            {"id": "9", "price": null},
            {"id": 10, "price": 1},
            {"price": 1},
            "garbage",
            {"id": "11", "price": 5}
        ]});
        let t = parse_snapshot(&body);
        assert_eq!(t.len(), 1);
        assert_eq!(t["11"], 5.0);
    }

    #[test]
    fn malformed_bodies_give_empty_table() {
        assert!(parse_snapshot(&json!({})).is_empty());
        assert!(parse_snapshot(&json!({"data": "x"})).is_empty());
        assert!(parse_snapshot(&json!({"data": {"id": "1", "price": 1}})).is_empty());
        assert!(parse_snapshot(&json!([1, 2, 3])).is_empty());
    }

    #[test]
    fn table_size_is_capped() {
        let rows: Vec<_> = (0..(MAX_TABLE_ENTRIES + 500))
            .map(|i| json!({"id": i.to_string(), "price": 1}))
            .collect();
        let t = parse_snapshot(&json!({ "data": rows }));
        assert_eq!(t.len(), MAX_TABLE_ENTRIES);
    }

    #[test]
    fn custom_price_url_must_be_https() {
        std::env::set_var("TLI_PRICE_API_URL", "http://evil.example/api");
        assert!(etor_bases().iter().all(|b| b.starts_with("https://")));
        assert!(!etor_bases().iter().any(|b| b.contains("evil")));
        std::env::set_var("TLI_PRICE_API_URL", "https://mirror.example/api/");
        assert_eq!(etor_bases()[0], "https://mirror.example/api");
        std::env::remove_var("TLI_PRICE_API_URL");
    }
}
