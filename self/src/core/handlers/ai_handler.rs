/*
HERE WE DEFINE THE LOGIC OF THE OPCODE AI.
CURRENTLY WE ARE USING THE OPENAI LLM
BUT WE COULD IN THE FUTURE IMPLEMENT ANOTHER
PROVIDER OR ENABLE USER IMPLEMENTATION OF
PROVIDER.
*/

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

pub fn ai_handler(args: Vec<String>, debug: bool) {
    let request = args[0].clone();
    let context = args[1].clone();
    if debug {
        println!("AI <- {}({})", request, context);
    }
    let prompt = format!(
    "Here is the English translation of your prompt:

---

#### INPUT 1

You are a machine embedded within another system. Your only function is to evaluate expressions and return a specific value. You must not reason out loud or provide explanations. Simply analyze the query and respond with a single value in the following format:

<value_type:value>


---

Valid value types:

* `bool`: for logical expressions (e.g. `true` or `false`)
* `string`: for text values
* `number`: for numeric values
* `nothing`: if you cannot determine a value type or if the expression produces no output

---

Inputs:

You are provided with two elements:

1. **query**: a string that describes the operation to perform, for example:
   \"print hello if <arg> is greater than 10\"

2. **context**: a dictionary of variables and their current values, for example:
   {{ \"arg\": 11 }}

Context variables may appear in the query enclosed in < >, and you must evaluate them correctly.

---

Rules:

* If the conditional expression is not met, respond with `nothing`.
* If there are no conditionals but you can infer the type and value, do so.
* If you cannot determine a type with certainty, respond with `nothing`.
* Never respond with any additional text. Only the final value.

---

Start execution with the following input:

**query**: {}
**context**: {{ \"arg\": {} }}
", request, context);

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
        println!("AI: (FAILED){}", res.status());
        return;
    }

    let response: ChatResponse = res.json().expect("AI: Failed to parse response");
    let answer = &response.choices[0].message.content;

    if debug {
        println!("AI -> {}", answer);
    }
}
