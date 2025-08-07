use std::{env, vec};

use reqwest::{blocking::Client, blocking::Response};

use crate::std::ai::providers::{ChatRequest, Message};

pub fn fetch(prompt: String) -> Response {
    let api_key = env::var("MISTRAL_API_KEY").expect("MISTRAL_API_KEY not set");

    let client = Client::new();
    let request_body = ChatRequest {
        model: "mistral-medium".to_string(), // O "mistral-small", "mistral-large", etc.
        messages: vec![Message {
            role: "system".to_string(),
            content: prompt,
        }],
    };

    let res = client
        .post("https://api.mistral.ai/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&request_body)
        .send()
        .expect("AI: Failed to send request");

    res
}
