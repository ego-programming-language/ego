pub mod fs_errors;
use crate::{core::error::fs_errors::FsError, opcodes::DataType, stack::OperandsStackValue};

pub enum VMErrorType {
    TypeCoercionError(OperandsStackValue), // maybe here we should have a more generic value, we'll see with time
    InvalidBinaryOperation(InvalidBinaryOperation),
    DivisionByZero(OperandsStackValue),
    UndeclaredIdentifierError(String),
    NotCallableError(String),
    Fs(FsError),
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
        VMErrorType::DivisionByZero(v) => {
            let source = if let Some(origin) = &v.origin {
                origin
            } else {
                &v.value.to_string()
            };

            (
                "Invalid division".to_string(),
                format!("Cannot devide {source} by 0",),
            )
        }
        VMErrorType::UndeclaredIdentifierError(v) => {
            ("Undeclared identifier".to_string(), format!("{}", v))
        }
        VMErrorType::NotCallableError(v) => ("Not callable member".to_string(), format!("{}", v)),
        VMErrorType::Fs(fs) => match fs {
            FsError::FileNotFound(s) => ("File not found".to_string(), format!("{}", s)),
            FsError::NotAFile(s) => ("Not a file".to_string(), format!("{}", s)),
            FsError::ReadError(s) => ("Read error".to_string(), format!("{}", s)),
        },
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
