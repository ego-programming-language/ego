use std::{env, vec};

use reqwest::{blocking::Client, blocking::Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub struct Choice {
    pub message: MessageContent,
}

#[derive(Deserialize)]
pub struct MessageContent {
    pub content: String,
}

pub fn fetch_ai(prompt: String) -> Response {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    let client = Client::new();
    let request_body = ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![Message {
            role: "system".to_string(),
            content: prompt,
        }],
    };

    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&request_body)
        .send()
        .expect("AI: Failed to send request");

    res
}
