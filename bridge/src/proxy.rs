//! HTTP proxy layer: shared state and request handling

use crate::config::BridgeConfig;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for the proxy
pub struct ProxyState {
    pub config: BridgeConfig,
    pub http_client: Client,
}

impl ProxyState {
    pub fn new(config: BridgeConfig) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }
}

pub type SharedProxyState = Arc<RwLock<ProxyState>>;
