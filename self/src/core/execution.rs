use crate::{types::Value, vm::Vm};

use super::error::{throw, VMError, VMErrorType};

pub struct VMExecutionResult {
    pub error: Option<VMError>,
    pub result: Option<Value>,
    //pub exports: Vec<Value>,
    // eventually here we could implement things like:
    // traceback
    // execution time
    // ...
}

impl VMExecutionResult {
    pub fn terminate(result: Option<Value>) -> VMExecutionResult {
        VMExecutionResult {
            error: None,
            result: result,
        }
    }

    pub fn terminate_with_errors(error_type: VMErrorType, vm: &Vm) -> VMExecutionResult {
        VMExecutionResult {
            error: Some(throw(error_type, vm)),
            result: None,
        }
    }
}
