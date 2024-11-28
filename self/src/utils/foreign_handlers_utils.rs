use crate::core::handlers::foreign_handlers::ForeignHandlersToml;
use std::env;
use std::fs;
use toml;

pub fn get_foreign_handlers() -> Option<ForeignHandlersToml> {
    let cwd = env::current_dir().unwrap_or_default();
    let file_path = cwd.join("foreign.toml");

    if !file_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&file_path).unwrap_or_default();
    let toml_content: ForeignHandlersToml = toml::from_str(&content).expect("Failed to parse TOML");
    Some(toml_content)
}
