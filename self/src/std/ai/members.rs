/*
HERE WE DEFINE THE LOGIC OF THE AI STD MODULE.
CURRENTLY WE ARE USING THE OPENAI LLM
BUT WE COULD IN THE FUTURE IMPLEMENT ANOTHER
PROVIDER OR ENABLE USER IMPLEMENTATION OF
PROVIDER.
*/

use std::{env, vec};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as SValue;

use crate::{
    core::error::{self, action_errors::ActionError, ai_errors::AIError, VMError, VMErrorType},
    heap::{HeapObject, HeapRef},
    std::{
        ai::types::Action, gen_native_modules_defs, generate_native_module, get_native_module_type,
        utils::cast_json_value, vector,
    },
    types::{
        object::{
            func::{Engine, Function},
            native_struct::NativeStruct,
            vector::Vector,
        },
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

fn get_response_json(response: &String) -> String {
    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    cleaned.to_string()
}

fn ai_response_parser(response: &String) -> Option<Value> {
    let cleaned = get_response_json(response);
    let json: SValue = serde_json::from_str(cleaned.as_str()).ok()?;
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

#[derive(Debug, Deserialize, Clone)]
struct AIAction {
    module: String,
    member: String,
    params: Vec<serde_json::Value>,
}

pub fn infer(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    let request_ref = params[0].clone();
    let request = match request_ref {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r);
            let request = match heap_obj {
                HeapObject::String(s) => s,
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(vm),
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
        Value::BoundAccess(_) => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: "bound_access".to_string(),
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
                        received: heap_obj.to_string(vm),
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
        Value::BoundAccess(_) => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: "bound_access".to_string(),
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

pub fn do_fn(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    let request_ref = params[0].clone();
    let request = match request_ref {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r);
            let request = match heap_obj {
                HeapObject::String(s) => s,
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(vm),
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
        Value::BoundAccess(_) => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: "bound_access".to_string(),
            }));
        }
    };

    if debug {
        println!("AI.DO <- {}", request);
    }

    let stdlib_defs: Vec<String> = gen_native_modules_defs()
        .iter()
        .map(|nm| nm.to_string())
        .collect();

    // we should try to avoid prompt injection
    // maybe using multiple prompts?
    let prompt = format!(
        "You are a virtual machine assistant with access to the following native modules:\n\n{}\n\n
        
You must respond to the following instruction with a list of JSON objects, where each object contains:

- 'module': the name of the module from the list above,
- 'member': the specific function name to call (from the members),
- 'params': an array of arguments.

You must only use the modules and members listed above. Do not invent anything.

Respond only with JSON. Do not include any explanations or markdown.

Instruction: {}",
        stdlib_defs.join("\n\n"),
        request
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
        .expect("AI.DO: Failed to send request");

    if !res.status().is_success() {
        println!("AI.DO (FAILED) -> {}", res.status());
        return Err(error::throw(VMErrorType::AI(AIError::AIFetchError(
            res.status().to_string(),
        ))));
    }

    let response: ChatResponse = res.json().expect("AI.DO: Failed to parse response");
    let answer = &response.choices[0].message.content;

    if debug {
        println!("AI -> {}", answer);
    }

    let cleaned = get_response_json(answer);
    let instructions: Vec<AIAction> = if let Ok(val) = serde_json::from_str(cleaned.as_str()) {
        val
    } else {
        return Ok(Value::RawValue(RawValue::Nothing));
    };
    if instructions.len() < 1 {
        return Ok(Value::RawValue(RawValue::Nothing));
    }

    // for the moment the function is allocated on
    // execution. but we should have a way of on a
    // native module import executed the generic code
    // to have things on scope, like, exec function.
    let exec_fn = Function::new("exec".to_string(), vec![], Engine::Native(exec));
    let exec_ref = vm.heap.allocate(HeapObject::Function(exec_fn));

    let actions: Vec<Action> = instructions
        .iter()
        .map(|instr| {
            Action::new(
                instr.module.clone(),
                exec_ref.clone(),
                instr.member.clone(),
                instr
                    .params
                    .iter()
                    .map(|p| {
                        if let Some(v) = cast_json_value(p) {
                            v
                        } else {
                            Value::RawValue(RawValue::Nothing)
                        }
                    })
                    .collect::<Vec<Value>>(),
            )
        })
        .collect();

    if debug {
        println!("AI.DO <- {:#?}", actions)
    }

    let mut actions_ref = vec![];
    for action in actions {
        actions_ref
            .push(Value::HeapRef(vm.heap.allocate(HeapObject::NativeStruct(
                NativeStruct::Action(action),
            ))));
    }

    // store all actions ref in a vector and return the
    // vector allocated heap ref
    let mut vector = Vector::new(actions_ref);
    vector::init_vector_members(&mut vector, vm);
    let vector_ref = vm.heap.allocate(HeapObject::Vector(vector));
    return Ok(Value::HeapRef(vector_ref));
}

pub fn exec(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    // resolve 'self'
    let (_self, _self_ref) = if let Some(_this) = _self {
        if let HeapObject::NativeStruct(NativeStruct::Action(ns)) =
            vm.resolve_heap_ref(_this.clone())
        {
            (ns, _this)
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    };

    if debug {
        println!("ACTION <- {}.{}", _self.module, _self.member);
    }
    let native_module_type = if let Some(nmt) = get_native_module_type(&_self.module) {
        nmt
    } else {
        return Err(error::throw(VMErrorType::Action(
            ActionError::InvalidModule(_self.module.clone()),
        )));
    };
    let native_module = generate_native_module(native_module_type);
    let fields = native_module.1;
    let member = if let Some(member) = fields.iter().find(|m| m.0 == _self.member) {
        member
    } else {
        return Err(error::throw(VMErrorType::Action(
            ActionError::InvalidMember {
                module: _self.module.clone(),
                member: _self.member.clone(),
            },
        )));
    };

    match &member.1 {
        HeapObject::Function(f) => {
            let execution = vm.run_function(&f.clone(), Some(_self_ref), _self.args.clone(), debug);
            if let Some(err) = execution.error {
                return Err(err);
            }
            if let Some(result) = execution.result {
                return Ok(result);
            }
            return Ok(Value::RawValue(RawValue::Nothing));
        }
        _ => {
            // TODO: use self-vm errors system
            // in principle this should not happen since
            // to the AI should arrive only valid callable
            // members from the stdlib modules
            panic!("error, member is not callable");
        }
    }
}
