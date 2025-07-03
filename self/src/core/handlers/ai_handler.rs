/*
HERE WE DEFINE THE LOGIC OF THE OPCODE AI.
CURRENTLY WE ARE USING THE OPENAI LLM
BUT WE COULD IN THE FUTURE IMPLEMENT ANOTHER
PROVIDER OR ENABLE USER IMPLEMENTATION OF
PROVIDER.
*/

use regex::Regex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

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
    let re = Regex::new(r"^(\w+):(.*)$").ok()?;
    let caps = re.captures(response)?;

    let value_type = &caps[1];
    let raw_value = &caps[2];

    let is_valid = match value_type {
        "bool" => matches!(raw_value, "true" | "false"),
        "string" => !raw_value.is_empty(),
        "number" => raw_value.parse::<f64>().is_ok(),
        "nothing" => raw_value.is_empty(),
        _ => false,
    };

    if is_valid {
        Some((value_type.to_string(), raw_value.to_string()))
    } else {
        None
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
Analyze the query and respond with a single value in the following format:

<value_type:value>

Valid value types:

* bool: for logical expressions (e.g. true or false)
* number: for numeric values
* nothing: if you cannot determine a value type or if the expression produces no output
* string: for string values that are non other possible values

Inputs:

You are provided with two elements:

query: a string that describes a condition for example:
   '<arg> is greater than 10'

context: a dictionary of variables and their current values, for example:
   {{ 'arg': 11 }}

Context variables appears in the query enclosed in < >, and you must evaluate them correctly.

Response rules: 

* For boolean or logical values use bool:true or bool:false.
* If the conditional expression is not met, respond with nothing.
* If there are no conditionals but you can infer the type and value, do so.
* If you cannot determine a type with certainty, respond with nothing.
* Never respond with any additional text. Only the final value.

Infer the following input: 

query: {request} 
context: {{ 'arg': {context} }}
"
    );

    let api_key = "unset-key"; //env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    let client = Client::new();
    let request_body = ChatRequest {
        model: "gpt-4".to_string(),
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
