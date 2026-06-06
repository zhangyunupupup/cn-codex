//! cn-codex-bridge: Local API bridge for Chinese LLM providers
//!
//! Translates OpenAI Responses API (/v1/responses) to Chat Completions API
//! (/v1/chat/completions) so that Codex CLI can work with Chinese LLM
//! providers (DeepSeek, Qwen, GLM, Kimi, etc.)

mod config;
mod converter;
mod proxy;
mod server;

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "cn-codex-bridge",
    version,
    about = "Local API bridge for Chinese LLM providers"
)]
struct Args {
    /// Port to listen on
    #[arg(long, default_value = "15721")]
    port: u16,

    /// Upstream Chat Completions API base URL
    #[arg(long)]
    upstream_url: String,

    /// Environment variable name for the API key
    #[arg(long)]
    api_key_env: String,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Init tracing
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(&args.log_level);
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Read API key from environment
    let api_key = std::env::var(&args.api_key_env).unwrap_or_else(|_| {
        eprintln!("Error: Environment variable {} not set", args.api_key_env);
        eprintln!("Please set it before starting the bridge:");
        eprintln!("  export {}=\"your-api-key\"", args.api_key_env);
        std::process::exit(1);
    });

    let config = config::BridgeConfig {
        port: args.port,
        upstream_url: args.upstream_url,
        api_key,
    };

    tracing::info!("cn-codex-bridge v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Listening on 127.0.0.1:{}", config.port);
    tracing::info!("Upstream: {}", config.upstream_url);

    if let Err(e) = server::run(config).await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
