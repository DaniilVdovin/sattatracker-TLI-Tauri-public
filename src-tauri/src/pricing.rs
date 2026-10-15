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

const DEFAULT_PRICE_API: &str = "http://115.231.176.101:8080/get";

impl PricingManager {
    fn price_url(&self) -> String {
        let base =
            std::env::var("TLI_PRICE_API_URL").unwrap_or_else(|_| DEFAULT_PRICE_API.to_string());
        format!("{}?season_id={}", base, self.season_id)
    }

    pub fn new(season_id: u32, cache_path: PathBuf) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            season_id,
            cache_path,
        }
    }

    pub async fn load_disk_cache(&self) {
        let data = match tokio::fs::read_to_string(&self.cache_path).await {
            Ok(d) => d,
            Err(_) => return,
        };
        let map: HashMap<String, f64> = match serde_json::from_str(&data) {
            Ok(m) => m,
            Err(_) => return,
        };
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

    pub async fn refresh_prices(&self) {
        let url = self.price_url();
        match self.client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(map) = json.as_object() {
                        {
                            let mut cache = PRICE_CACHE.lock().unwrap();
                            for (id, item_data) in map {
                                if let Some(price) = item_data.get("price").and_then(|p| p.as_f64())
                                {
                                    cache.insert(id.to_string(), price);
                                } else if let Some(price_str) =
                                    item_data.get("price").and_then(|p| p.as_str())
                                {
                                    if let Ok(price) = price_str.parse::<f64>() {
                                        cache.insert(id.to_string(), price);
                                    }
                                }
                            }
                        }
                        FULL_PRICE_LOADED.store(true, Ordering::Relaxed);
                        self.save_disk_cache().await;
                        println!("[pricing] Price table refreshed from API and saved to disk");
                    }
                }
            }
            Err(e) => {
                eprintln!("[pricing] API Error (keeping last known prices): {}", e);
                FULL_PRICE_LOADED.store(true, Ordering::Relaxed);
            }
        }
    }

    async fn fetch_price_from_api(&self, item_id: &str) -> f64 {
        let url = self.price_url();

        match self.client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(map) = json.as_object() {
                        {
                            let mut cache = PRICE_CACHE.lock().unwrap();
                            for (id, item_data) in map {
                                if let Some(price) = item_data.get("price").and_then(|p| p.as_f64())
                                {
                                    cache.insert(id.to_string(), price);
                                } else if let Some(price_str) =
                                    item_data.get("price").and_then(|p| p.as_str())
                                {
                                    if let Ok(price) = price_str.parse::<f64>() {
                                        cache.insert(id.to_string(), price);
                                    }
                                }
                            }
                        }
                        FULL_PRICE_LOADED.store(true, Ordering::Relaxed);
                        self.save_disk_cache().await;
                        println!("[pricing] Price table fetched from API and saved to disk");
                    }
                }
            }
            Err(e) => {
                eprintln!("[pricing] API Error (using last known prices): {}", e);
                FULL_PRICE_LOADED.store(true, Ordering::Relaxed);
            }
        }

        let cache = PRICE_CACHE.lock().unwrap();
        if let Some(&price) = cache.get(item_id) {
            price
        } else {
            0.0
        }
    }
}
