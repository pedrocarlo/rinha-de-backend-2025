use std::sync::Arc;

use url::Url;

pub const DEFAULT_URL_ENV_NAME: &str = "PROCESSOR_DEFAULT_URL";
pub const FALLBACK_URL_ENV_NAME: &str = "PROCESSOR_FALLBACK_URL";
pub const REDIS_URL_ENV_NAME: &str = "REDIS_URL";

/// Stores Clients and Urls
#[derive(Debug, Clone)]
pub struct AppState {
    pub default_url: Arc<Url>,
    pub fallback_url: Arc<Url>,
    pub http_client: reqwest::Client,
    pub redis_client: redis::Client,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            default_url: Arc::new(
                Url::parse(&std::env::var(DEFAULT_URL_ENV_NAME).unwrap()).unwrap(),
            ),
            fallback_url: Arc::new(
                Url::parse(&std::env::var(FALLBACK_URL_ENV_NAME).unwrap()).unwrap(),
            ),
            http_client: reqwest::ClientBuilder::new()
                .pool_max_idle_per_host(1000) // Max idle connections per host
                .build()
                .unwrap(),
            redis_client: redis::Client::open(std::env::var(REDIS_URL_ENV_NAME).unwrap()).unwrap(),
        }
    }
}
