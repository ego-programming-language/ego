use super::error::{throw, VMError, VMErrorType};

pub struct VMExecutionResult {
    pub error: Option<VMError>,
    // eventually here we could implement things like:
    // traceback
    // execution time
    // ...
}

impl VMExecutionResult {
    pub fn terminate() -> VMExecutionResult {
        VMExecutionResult { error: None }
    }

    pub fn terminate_with_errors(error_type: VMErrorType) -> VMExecutionResult {
        VMExecutionResult {
            error: Some(throw(error_type)),
        }
    }
}
