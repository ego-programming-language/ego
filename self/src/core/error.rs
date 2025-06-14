use crate::{opcodes::DataType, vm::StackValue};

pub enum VMErrorType {
    TypeCoercionError(StackValue), // maybe here we should have a more generic value, we'll see with time
    InvalidBinaryOperation(InvalidBinaryOperation),
}

pub struct VMError {
    pub error_type: VMErrorType,
    pub message: String,
    pub semantic_message: String,
}

pub fn throw(error_type: VMErrorType) -> VMError {
    let error = match &error_type {
        VMErrorType::TypeCoercionError(v) => {
            let source = if let Some(origin) = &v.origin {
                origin
            } else {
                &v.value.to_string()
            };
            (
                "Type coercion error".to_string(),
                format!(
                    "implicit conversion is not permitted. Problem with {}",
                    source
                ),
            )
        }
        VMErrorType::InvalidBinaryOperation(v) => (
            "Invalid binary operation".to_string(),
            format!("{} {} {}", v.left.as_str(), v.operator, v.right.as_str()),
        ),
    };

    VMError {
        error_type: error_type,
        message: error.0,
        semantic_message: error.1,
    }
}

pub struct InvalidBinaryOperation {
    pub left: DataType,
    pub right: DataType,
    pub operator: String,
}
