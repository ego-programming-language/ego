use std::env;

use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

use crate::core::error::{ai_errors::AIError, VMErrorType};

mod mistral;
mod openai;

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

pub fn fetch_ai(prompt: String) -> Result<Response, VMErrorType> {
    let ai_engine = env::var("SELF_AI_ENGINE");
    let ai_engine = if let Ok(engine) = ai_engine {
        engine
    } else {
        return Err(VMErrorType::AI(AIError::AIEngineNotSet()));
    };

    match ai_engine.as_str() {
        "openai" => Ok(openai::fetch(prompt)),
        "mistral" => Ok(mistral::fetch(prompt)),
        _ => Err(VMErrorType::AI(AIError::AIEngineNotImplemented(ai_engine))),
    }
}
