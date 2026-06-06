//! Bridge configuration

use serde::{Deserialize, Serialize};

/// Bridge server configuration
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Local port to listen on
    pub port: u16,
    /// Upstream Chat Completions API base URL
    pub upstream_url: String,
    /// API key for the upstream provider
    pub api_key: String,
}

/// Provider configuration file format (TOML)
#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub name: String,
    pub base_url: String,
    pub env_key: String,
    pub models: Vec<ModelEntry>,
    #[serde(default)]
    pub default_model: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct ModelEntry {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl BridgeConfig {
    /// Get the upstream chat completions endpoint URL
    pub fn chat_completions_url(&self) -> String {
        let base = self.upstream_url.trim_end_matches('/');
        format!("{}/chat/completions", base)
    }

    /// Get the upstream models endpoint URL
    #[allow(dead_code)]
    pub fn models_url(&self) -> String {
        let base = self.upstream_url.trim_end_matches('/');
        format!("{}/models", base)
    }
}
