pub fn infer_prompt(request: &String, context: &String) -> String {
    return format!(
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
}

pub fn do_prompt(stdlib_defs: Vec<String>, request: &String) -> String {
    return format!(
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
}
