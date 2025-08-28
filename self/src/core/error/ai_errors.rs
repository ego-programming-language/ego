#[derive(Debug)]
pub enum AIError {
    AIFetchError(String),
    AIEngineNotSet(),
    AIEngineNotImplemented(String),
}
