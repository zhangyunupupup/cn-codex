//! Protocol converter: Responses API <-> Chat Completions API
//!
//! This module handles the bidirectional translation between OpenAI's Responses API
//! format (used by Codex CLI) and the Chat Completions API format (used by Chinese
//! LLM providers like DeepSeek, Qwen, GLM, etc.)

use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================
// Responses API -> Chat Completions Request Conversion
// ============================================================

/// Convert a Responses API request body to a Chat Completions request body
pub fn responses_to_chat_request(responses_body: &Value) -> Result<Value, ConverterError> {
    let model = responses_body["model"]
        .as_str()
        .ok_or(ConverterError::MissingField("model"))?;

    let input = &responses_body["input"];
    let instructions = responses_body["instructions"].as_str().unwrap_or("");

    // Convert input items to messages
    let mut messages = Vec::new();

    // Add system message from instructions
    if !instructions.is_empty() {
        messages.push(json!({
            "role": "system",
            "content": instructions
        }));
    }

    // Process input items
    if let Some(items) = input.as_array() {
        for item in items {
            if let Some(msg) = convert_input_item_to_message(item) {
                messages.push(msg);
            }
        }
    } else if let Some(content) = input.as_str() {
        // Simple string input
        messages.push(json!({
            "role": "user",
            "content": content
        }));
    }

    // Convert tools
    let tools = convert_tools(&responses_body["tools"]);

    // Build Chat Completions request
    let mut chat_request = json!({
        "model": model,
        "messages": messages,
        "stream": responses_body["stream"].as_bool().unwrap_or(true),
    });

    // Add tools if present
    if !tools.is_empty() {
        chat_request["tools"] = json!(tools);
    }

    // Add tool_choice if present
    if let Some(tool_choice) = responses_body.get("tool_choice") {
        chat_request["tool_choice"] = tool_choice.clone();
    }

    // Add temperature (default for chat models)
    if let Some(temp) = responses_body.get("temperature") {
        chat_request["temperature"] = temp.clone();
    }

    // Add max_tokens if specified
    if let Some(max_tokens) = responses_body.get("max_output_tokens") {
        chat_request["max_tokens"] = max_tokens.clone();
    }

    Ok(chat_request)
}

/// Convert a single Responses API input item to a Chat Completions message
fn convert_input_item_to_message(item: &Value) -> Option<Value> {
    let item_type = item["type"].as_str().unwrap_or("");

    match item_type {
        "message" => {
            let role = item["role"].as_str().unwrap_or("user");
            let content = &item["content"];

            // Handle content array (multiple content parts)
            let chat_content = if let Some(content_arr) = content.as_array() {
                let mut parts = Vec::new();
                for part in content_arr {
                    let part_type = part["type"].as_str().unwrap_or("");
                    match part_type {
                        "input_text" | "output_text" => {
                            parts.push(json!({
                                "type": "text",
                                "text": part["text"]
                            }));
                        }
                        "input_image" => {
                            if let Some(url) = part["image_url"].as_str() {
                                parts.push(json!({
                                    "type": "image_url",
                                    "image_url": { "url": url }
                                }));
                            }
                        }
                        _ => {
                            // Try to extract text from other content types
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                parts.push(json!({
                                    "type": "text",
                                    "text": text
                                }));
                            }
                        }
                    }
                }
                if parts.len() == 1 {
                    Value::String(parts[0]["text"].as_str().unwrap_or("").to_string())
                } else {
                    json!(parts)
                }
            } else if let Some(text) = content.as_str() {
                Value::String(text.to_string())
            } else {
                json!(content)
            };

            Some(json!({
                "role": role,
                "content": chat_content
            }))
        }
        "function_call" => {
            // Tool call from assistant
            let call_id = item["call_id"].as_str().unwrap_or("");
            let name = item["name"].as_str().unwrap_or("");
            let arguments = item["arguments"].as_str().unwrap_or("{}");

            Some(json!({
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": call_id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": arguments
                    }
                }]
            }))
        }
        "function_call_output" => {
            // Tool result
            let call_id = item["call_id"].as_str().unwrap_or("");
            let output = item["output"].as_str().unwrap_or("");

            Some(json!({
                "role": "tool",
                "tool_call_id": call_id,
                "content": output
            }))
        }
        _ => None,
    }
}

/// Convert Responses API tools to Chat Completions tools
fn convert_tools(tools_value: &Value) -> Vec<Value> {
    let mut result = Vec::new();

    if let Some(tools) = tools_value.as_array() {
        for tool in tools {
            let tool_type = tool["type"].as_str().unwrap_or("");
            match tool_type {
                "function" => {
                    result.push(json!({
                        "type": "function",
                        "function": {
                            "name": tool["name"],
                            "description": tool.get("description").unwrap_or(&json!("")),
                            "parameters": tool.get("parameters").unwrap_or(&json!({})),
                        }
                    }));
                }
                _ => {
                    // Pass through other tool types
                    result.push(tool.clone());
                }
            }
        }
    }

    result
}

// ============================================================
// Chat Completions -> Responses API Response/Stream Conversion
// ============================================================

/// Convert a non-streaming Chat Completions response to a Responses API response
pub fn chat_to_responses_response(
    chat_response: &Value,
    model: &str,
) -> Result<Value, ConverterError> {
    let id = chat_response["id"]
        .as_str()
        .unwrap_or("resp_bridge")
        .replace("chatcmpl-", "resp_");

    let choices = chat_response["choices"]
        .as_array()
        .ok_or(ConverterError::MissingField("choices"))?;

    let mut output = Vec::new();

    for choice in choices {
        let message = &choice["message"];

        // Add text content if present
        if let Some(content) = message["content"].as_str() {
            if !content.is_empty() {
                output.push(json!({
                    "type": "message",
                    "role": "assistant",
                    "content": [{
                        "type": "output_text",
                        "text": content
                    }]
                }));
            }
        }

        // Add tool calls if present
        if let Some(tool_calls) = message["tool_calls"].as_array() {
            for tc in tool_calls {
                output.push(json!({
                    "type": "function_call",
                    "call_id": tc["id"],
                    "name": tc["function"]["name"],
                    "arguments": tc["function"]["arguments"],
                }));
            }
        }
    }

    let usage = &chat_response["usage"];

    Ok(json!({
        "id": id,
        "object": "response",
        "model": model,
        "output": output,
        "usage": {
            "input_tokens": usage["prompt_tokens"].as_u64().unwrap_or(0),
            "output_tokens": usage["completion_tokens"].as_u64().unwrap_or(0),
        },
        "status": "completed"
    }))
}

/// State machine for converting streaming Chat Completions chunks to Responses SSE events
pub struct StreamConverter {
    response_id: String,
    model: String,
    output_index: usize,
    content_index: usize,
    current_tool_calls: HashMap<usize, ToolCallState>,
}

struct ToolCallState {
    id: String,
    name: String,
    arguments: String,
}

impl StreamConverter {
    pub fn new(model: String) -> Self {
        Self {
            response_id: format!("resp_br_{}", uuid_short()),
            model,
            output_index: 0,
            content_index: 0,
            current_tool_calls: HashMap::new(),
        }
    }

    /// Convert a streaming Chat Completions chunk to Responses API SSE events
    pub fn convert_chunk(&mut self, chunk: &Value) -> Vec<Value> {
        let mut events = Vec::new();

        // First chunk: response.created
        if self.output_index == 0 && self.current_tool_calls.is_empty() {
            events.push(self.make_created_event());
        }

        let choices = match chunk["choices"].as_array() {
            Some(c) => c,
            None => return events,
        };

        for choice in choices {
            let delta = &choice["delta"];
            let finish_reason = choice["finish_reason"].as_str().unwrap_or("");

            // Text content delta
            if let Some(content) = delta["content"].as_str() {
                if !content.is_empty() {
                    if self.output_index == 0
                        || (self.current_tool_calls.is_empty() && self.output_index <= 1)
                    {
                        // First text: emit output_item.added
                        if self.output_index == 0 {
                            events.push(self.make_text_item_added_event());
                            self.output_index += 1;
                        }
                    }
                    events.push(self.make_text_delta_event(content));
                }
            }

            // Tool call deltas
            if let Some(tool_calls) = delta["tool_calls"].as_array() {
                for tc in tool_calls {
                    let idx = tc["index"].as_u64().unwrap_or(0) as usize;

                    let tc_state =
                        self.current_tool_calls
                            .entry(idx)
                            .or_insert_with(|| ToolCallState {
                                id: tc["id"]
                                    .as_str()
                                    .unwrap_or(&format!("call_{}", idx))
                                    .to_string(),
                                name: tc["function"]["name"].as_str().unwrap_or("").to_string(),
                                arguments: String::new(),
                            });

                    // Append function arguments delta
                    if let Some(args_delta) = tc["function"]["arguments"].as_str() {
                        tc_state.arguments.push_str(args_delta);
                    }

                    // Extract id before borrowing self again
                    let call_id = tc_state.id.clone();
                    let args_d = args_delta(tc).to_string();

                    // Emit function_call arguments delta
                    events.push(json!({
                        "type": "response.function_call_arguments.delta",
                        "item_id": call_id,
                        "delta": args_d
                    }));
                }
            }

            // Finish reason
            if !finish_reason.is_empty() && finish_reason != "null" {
                // Complete any pending tool calls
                for tc_state in self.current_tool_calls.values() {
                    events.push(self.make_function_call_done_event(
                        &tc_state.id,
                        &tc_state.name,
                        &tc_state.arguments,
                    ));
                }
                self.current_tool_calls.clear();

                events.push(self.make_completed_event(finish_reason));
            }
        }

        events
    }

    fn make_created_event(&self) -> Value {
        json!({
            "type": "response.created",
            "response": {
                "id": self.response_id,
                "object": "response",
                "status": "in_progress",
                "model": self.model,
                "output": []
            }
        })
    }

    fn make_text_item_added_event(&self) -> Value {
        json!({
            "type": "response.output_item.added",
            "output_index": 0,
            "item": {
                "type": "message",
                "role": "assistant",
                "content": []
            }
        })
    }

    fn make_text_delta_event(&self, text: &str) -> Value {
        json!({
            "type": "response.output_text.delta",
            "output_index": 0,
            "content_index": self.content_index,
            "delta": text
        })
    }

    #[allow(dead_code)]
    fn make_function_call_delta_event(&self, call_id: &str, args_delta: &str) -> Value {
        json!({
            "type": "response.function_call_arguments.delta",
            "item_id": call_id,
            "delta": args_delta
        })
    }

    fn make_function_call_done_event(&self, call_id: &str, name: &str, arguments: &str) -> Value {
        json!({
            "type": "response.output_item.done",
            "output_index": self.output_index,
            "item": {
                "type": "function_call",
                "call_id": call_id,
                "name": name,
                "arguments": arguments
            }
        })
    }

    fn make_completed_event(&self, finish_reason: &str) -> Value {
        let status = match finish_reason {
            "tool_calls" => "incomplete", // needs tool output
            _ => "completed",
        };

        json!({
            "type": "response.completed",
            "response": {
                "id": self.response_id,
                "status": status,
            }
        })
    }
}

fn args_delta(tc: &Value) -> &str {
    tc["function"]["arguments"].as_str().unwrap_or("")
}

fn uuid_short() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}", dur.as_millis())
}

// ============================================================
// Error Types
// ============================================================

#[derive(Debug, thiserror::Error)]
pub enum ConverterError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    #[allow(dead_code)]
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_responses_to_chat_simple() {
        let responses = json!({
            "model": "deepseek-chat",
            "instructions": "You are a helpful assistant.",
            "input": [
                {
                    "type": "message",
                    "role": "user",
                    "content": "Hello!"
                }
            ],
            "stream": true
        });

        let chat = responses_to_chat_request(&responses).unwrap();

        assert_eq!(chat["model"], "deepseek-chat");
        assert_eq!(chat["stream"], true);

        let messages = chat["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 2); // system + user
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[1]["role"], "user");
    }

    #[test]
    fn test_responses_to_chat_with_tools() {
        let responses = json!({
            "model": "deepseek-chat",
            "input": [
                {
                    "type": "message",
                    "role": "user",
                    "content": "Read the file main.rs"
                }
            ],
            "tools": [{
                "type": "function",
                "name": "read_file",
                "description": "Read a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    }
                }
            }],
            "stream": true
        });

        let chat = responses_to_chat_request(&responses).unwrap();
        let tools = chat["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["function"]["name"], "read_file");
    }

    #[test]
    fn test_chat_to_responses() {
        let chat_resp = json!({
            "id": "chatcmpl-123",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I help?"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5
            }
        });

        let resp = chat_to_responses_response(&chat_resp, "deepseek-chat").unwrap();
        assert_eq!(resp["object"], "response");
        assert_eq!(resp["status"], "completed");
    }

    #[test]
    fn test_stream_converter() {
        let mut converter = StreamConverter::new("deepseek-chat".to_string());

        let chunk = json!({
            "choices": [{
                "delta": { "content": "Hello" },
                "finish_reason": null
            }]
        });

        let events = converter.convert_chunk(&chunk);
        assert!(!events.is_empty());
        // Should have created event + text item added + text delta
        assert!(events.len() >= 2);
    }
}
