/*
HERE WE DEFINE THE LOGIC OF THE AI STD MODULE.
CURRENTLY WE ARE USING THE OPENAI LLM
BUT WE COULD IN THE FUTURE IMPLEMENT ANOTHER
PROVIDER OR ENABLE USER IMPLEMENTATION OF
PROVIDER.
*/

use std::env;

use regex::Regex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as SValue;

use crate::{
    core::error::{self, ai_errors::AIError, VMError, VMErrorType},
    heap::HeapObject,
    types::{
        raw::{bool::Bool, f64::F64, utf8::Utf8, RawValue},
        Value,
    },
    vm::Vm,
};

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

fn ai_response_parser(response: &String) -> Option<Value> {
    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let json: SValue = serde_json::from_str(cleaned).ok()?;
    let raw_value = json.get("value")?;

    if raw_value.is_boolean() {
        let bool = raw_value.as_bool().unwrap();
        Some(Value::RawValue(RawValue::Bool(Bool::new(bool))))
    } else if raw_value.is_number() {
        let value = raw_value.as_f64().unwrap();
        Some(Value::RawValue(RawValue::F64(F64::new(value))))
    } else if raw_value.is_string() {
        let s = raw_value.as_str()?;
        if s.trim().is_empty() || s.trim() == "nothing" {
            Some(Value::RawValue(RawValue::Nothing))
        } else {
            let value = raw_value.as_str().unwrap();
            Some(Value::RawValue(RawValue::Utf8(Utf8::new(
                value.to_string(),
            ))))
        }
    } else {
        Some(Value::RawValue(RawValue::Nothing))
    }
}

pub fn infer(vm: &mut Vm, params: Vec<Value>, debug: bool) -> Result<Value, VMError> {
    let request_ref = params[0].clone();
    let request = match request_ref {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r);
            let request = match heap_obj {
                HeapObject::String(s) => s,
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(),
                    }));
                }
            };
            request
        }
        Value::RawValue(r) => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: r.get_type_string(),
            }));
        }
    };
    let context_ref = params[1].clone();
    let context = match context_ref {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r);
            let context = match heap_obj {
                HeapObject::String(s) => s,
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(),
                    }));
                }
            };
            context
        }
        Value::RawValue(r) => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: r.get_type_string(),
            }));
        }
    };

    if debug {
        println!("AI <- {}({})", request, context.to_string());
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

query: {} 
context: {{ 'arg': {} }}
",
        request.to_string(),
        context.to_string()
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
        return Err(error::throw(VMErrorType::AI(AIError::AIFetchError(
            res.status().to_string(),
        ))));
    }

    let response: ChatResponse = res.json().expect("AI: Failed to parse response");
    let answer = &response.choices[0].message.content;

    if debug {
        println!("AI -> {}", answer);
    }

    let parsed_answer = ai_response_parser(answer);
    if let Some(v) = parsed_answer {
        return Ok(v);
    } else {
        return Ok(Value::RawValue(RawValue::Nothing));
    }
}
