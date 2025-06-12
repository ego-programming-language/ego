use super::error::{throw, VMError, VMErrorType};

pub struct VMExecutionResult {
    error: Option<VMError>,
    // eventually here we could implement things like:
    // traceback
    // execution time
    // ...
}

impl VMExecutionResult {
    pub fn terminate() -> VMExecutionResult {
        VMExecutionResult { error: None }
    }

    pub fn terminate_with_errors(
        error_type: VMErrorType,
        error_message: String,
    ) -> VMExecutionResult {
        VMExecutionResult {
            error: Some(throw(error_type, error_message)),
        }
    }
}
