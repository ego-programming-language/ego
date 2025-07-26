pub mod ai_errors;
pub mod fs_errors;
pub mod net_errors;
use crate::{
    core::error::{ai_errors::AIError, fs_errors::FsError, net_errors::NetErrors},
    opcodes::DataType,
    stack::OperandsStackValue,
};

pub enum VMErrorType {
    TypeCoercionError(OperandsStackValue), // maybe here we should have a more generic value, we'll see with time
    TypeMismatch { expected: String, received: String },
    InvalidBinaryOperation(InvalidBinaryOperation),
    DivisionByZero(OperandsStackValue),
    UndeclaredIdentifierError(String),
    NotCallableError(String),
    ModuleNotFound(String),
    ExportInvalidMemberType,
    Fs(FsError),
    AI(AIError),
    Net(NetErrors),
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
        VMErrorType::TypeMismatch { expected, received } => (
            "Type mismatch error".to_string(),
            format!("expected {expected}, received {received}"),
        ),
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
        VMErrorType::ModuleNotFound(s) => ("Module not found".to_string(), format!("{}", s)),
        VMErrorType::ExportInvalidMemberType => (
            "Export invalid member type".to_string(),
            format!("expected type <identifier> provided"),
        ),
        VMErrorType::Fs(fs) => match fs {
            FsError::FileNotFound(s) => ("File not found".to_string(), format!("{}", s)),
            FsError::NotAFile(s) => ("Not a file".to_string(), format!("{}", s)),
            FsError::ReadError(s) => ("Read error".to_string(), format!("{}", s)),
        },
        VMErrorType::AI(ai) => match ai {
            AIError::AIFetchError(s) => ("AI fetch error".to_string(), format!("{}", s)),
        },
        VMErrorType::Net(net) => match net {
            NetErrors::NetConnectError(s) => {
                ("Network connection error".to_string(), format!("{}", s))
            }
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
