use std::{env, vec};

use reqwest::{blocking::Client, blocking::Response};

use crate::std::ai::providers::{ChatRequest, Message};

pub fn fetch(prompt: String) -> Response {
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
