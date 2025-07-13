use std::{ops::Deref, sync::Arc};

use url::Url;

const DEFAULT_URL_ENV_NAME: &str = "PROCESSOR_DEFAULT_URL";
const FALLBACK_URL_ENV_NAME: &str = "PROCESSOR_FALLBACK_URL";
const REDIS_URL_ENV_NAME: &str = "REDIS_URL";

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AppStateInner::new()),
        }
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

///
#[derive(Debug)]
pub struct AppStateInner {
    pub default_url: Url,
    pub fallback_url: Url,
    pub http_client: reqwest::Client,
    pub redis_client: redis::Client,
}

impl AppStateInner {
    pub fn new() -> Self {
        Self {
            default_url: Url::parse(&std::env::var(DEFAULT_URL_ENV_NAME).unwrap()).unwrap(),
            fallback_url: Url::parse(&std::env::var(FALLBACK_URL_ENV_NAME).unwrap()).unwrap(),
            http_client: reqwest::Client::new(),
            redis_client: redis::Client::open(std::env::var(REDIS_URL_ENV_NAME).unwrap()).unwrap(),
        }
    }
}
