/*
HERE WE DEFINE THE LOGIC OF THE OPCODE AI.
CURRENTLY WE ARE USING THE OPENAI LLM
BUT WE COULD IN THE FUTURE IMPLEMENT ANOTHER
PROVIDER OR ENABLE USER IMPLEMENTATION OF
PROVIDER.
*/

use std::env;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

fn ai_response_parser(response: &String) -> Option<(String, String)> {
    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let json: Value = serde_json::from_str(cleaned).ok()?;
    let raw_value = json.get("value")?;

    if raw_value.is_boolean() {
        Some(("bool".to_string(), raw_value.to_string()))
    } else if raw_value.is_number() {
        Some(("number".to_string(), raw_value.to_string()))
    } else if raw_value.is_string() {
        let s = raw_value.as_str()?;
        if s.trim().is_empty() || s.trim() == "nothing" {
            Some(("nothing".to_string(), "nothing".to_string()))
        } else {
            Some(("string".to_string(), s.to_string()))
        }
    } else {
        Some(("nothing".to_string(), "nothing".to_string()))
    }
}

pub fn ai_handler(args: Vec<String>, debug: bool) -> Option<(String, String)> {
    let request = args[0].clone();
    let context = args[1].clone();
    if debug {
        println!("AI <- {}({})", request, context);
    }
    // we should try to avoid prompt injection
    // maybe using multiple prompts?
    let prompt = format!(
        "
Analyze the query and respond with a single value in the following json format:

{{
  \"value\": response-value
}}

You are provided two elements:

query: a string that describes a condition for example:
   '<arg> is greater than 10'

context: a dictionary of variables and their current values, for example:
   {{ 'arg': 11 }}

Context variables appears in the query enclosed in < >, and you must evaluate them correctly.

Response rules: 

* For boolean or logical values use true or false.
* If the conditional expression is not met, respond with nothing.
* If there are no conditionals but you can infer the type and value, do so.
* If you cannot determine a type with certainty, respond with nothing.
* Never respond with any additional text. Only the final value.

Infer the following input: 

query: {request} 
context: {{ 'arg': {context} }}
"
    );

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

    if !res.status().is_success() {
        println!("AI (FAILED) -> {}", res.status());
        return None;
    }

    let response: ChatResponse = res.json().expect("AI: Failed to parse response");
    let answer = &response.choices[0].message.content;

    if debug {
        println!("AI -> {}", answer);
    }

    let parsed_answer = ai_response_parser(answer);
    if let Some(v) = parsed_answer {
        Some((v.0, v.1))
    } else {
        None
    }
}
