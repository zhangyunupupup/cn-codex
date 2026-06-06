//! HTTP server for the bridge

use crate::config::BridgeConfig;
use crate::converter;
use crate::proxy::{ProxyState, SharedProxyState};
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use futures::StreamExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, instrument, warn};

/// Build and run the bridge server
pub async fn run(config: BridgeConfig) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(RwLock::new(ProxyState::new(config.clone())));

    let app = Router::new()
        .route("/v1/responses", post(handle_responses))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], config.port));
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "ok"
}

/// Handle Responses API requests (from Codex CLI)
#[instrument(skip(state, body))]
async fn handle_responses(
    State(state): State<SharedProxyState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    info!("Received Responses API request");

    let state_guard = state.read().await;
    let model = body["model"]
        .as_str()
        .unwrap_or("deepseek-chat")
        .to_string();

    // Convert request
    let chat_request = match converter::responses_to_chat_request(&body) {
        Ok(req) => req,
        Err(e) => {
            error!("Conversion error: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": { "message": format!("Request conversion failed: {}", e), "type": "bridge_error" }
                })),
            )
                .into_response();
        }
    };

    let is_stream = chat_request["stream"].as_bool().unwrap_or(true);

    // Build upstream request
    let url = state_guard.config.chat_completions_url();
    let api_key = &state_guard.config.api_key;

    let mut request_builder = state_guard
        .http_client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json");

    // Provider-specific headers
    let upstream_url = &state_guard.config.upstream_url;
    if upstream_url.contains("dashscope") && is_stream {
        request_builder = request_builder.header("X-DashScope-SSE", "enable");
    }

    let upstream_response = match request_builder.json(&chat_request).send().await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Upstream error: {}", e);
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": { "message": format!("Upstream error: {}", e), "type": "bridge_error" }
                })),
            )
                .into_response();
        }
    };

    let status = upstream_response.status();
    if !status.is_success() {
        let error_body = upstream_response.text().await.unwrap_or_default();
        error!(status = %status, body = %error_body, "Upstream error");
        return (
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            error_body,
        )
            .into_response();
    }

    if is_stream {
        // Streaming: convert Chat Completions SSE to Responses API SSE
        info!("Starting streaming response conversion");

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<String, std::io::Error>>(100);
        let stream_converter =
            Arc::new(tokio::sync::Mutex::new(converter::StreamConverter::new(model)));

        // Spawn background task to process the stream
        tokio::spawn(async move {
            let mut stream = upstream_response.bytes_stream();
            let mut buffer = String::new();
            let converter = stream_converter;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let text = String::from_utf8_lossy(&chunk);
                        buffer.push_str(&text);

                        // Process complete SSE events (separated by double newline)
                        while let Some(pos) = buffer.find("\n\n") {
                            let event_text = buffer[..pos].to_string();
                            buffer = buffer[pos + 2..].to_string();

                            if let Some(data) = parse_sse_data(&event_text) {
                                if data == "[DONE]" {
                                    // Send completed event
                                    let done_event = json!({
                                        "type": "response.completed",
                                        "response": { "status": "completed" }
                                    });
                                    let _ = tx
                                        .send(Ok(format_sse_line(&done_event)))
                                        .await;
                                    return;
                                }

                                if let Ok(chunk_data) = serde_json::from_str::<Value>(&data) {
                                    let mut conv = converter.lock().await;
                                    let events = conv.convert_chunk(&chunk_data);
                                    for event in events {
                                        let _ = tx.send(Ok(format_sse_line(&event))).await;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Stream read error: {}", e);
                        let _ = tx
                            .send(Err(std::io::Error::new(
                                std::io::ErrorKind::BrokenPipe,
                                e.to_string(),
                            )))
                            .await;
                        return;
                    }
                }
            }
        });

        // Convert mpsc receiver to axum body stream
        let body_stream = tokio_stream::wrappers::ReceiverStream::new(rx).map(|result| {
            result.map(|s| bytes::Bytes::from(s))
        });

        let body = Body::from_stream(body_stream);

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("text/event-stream"));
        headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));

        (StatusCode::OK, headers, body).into_response()
    } else {
        // Non-streaming: convert entire response
        let chat_response: Value = match upstream_response.json().await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse upstream response: {}", e);
                return (StatusCode::BAD_GATEWAY, "Upstream response parse error").into_response();
            }
        };

        let responses_response = match converter::chat_to_responses_response(&chat_response, &model)
        {
            Ok(r) => r,
            Err(e) => {
                error!("Response conversion error: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Response conversion error")
                    .into_response();
            }
        };

        (StatusCode::OK, Json(responses_response)).into_response()
    }
}

/// Format a Responses API event as an SSE line
fn format_sse_line(event: &Value) -> String {
    format!("event: response.done\ndata: {}\n\n", event)
}

/// Parse data field from SSE text
fn parse_sse_data(sse_text: &str) -> Option<String> {
    for line in sse_text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            let trimmed = data.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}
