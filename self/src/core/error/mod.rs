pub mod action_errors;
pub mod ai_errors;
pub mod fs_errors;
pub mod net_errors;
pub mod os_errors;
pub mod struct_errors;
pub mod type_errors;

use crate::{
    core::error::{
        action_errors::ActionError, ai_errors::AIError, fs_errors::FsError, net_errors::NetErrors,
        os_errors::OsError, struct_errors::StructError, type_errors::TypeError,
    },
    opcodes::DataType,
    stack::OperandsStackValue,
    vm::Vm,
};

#[derive(Debug)]
pub enum VMErrorType {
    TypeCoercionError(OperandsStackValue), // maybe here we should have a more generic value, we'll see with time
    TypeMismatch { expected: String, received: String },
    TypeError(TypeError),
    InvalidBinaryOperation(InvalidBinaryOperation),
    DivisionByZero(OperandsStackValue),
    UndeclaredIdentifierError(String),
    NotCallableError(String),
    ModuleNotFound(String),
    ExportInvalidMemberType,
    Fs(FsError),
    Os(OsError),
    AI(AIError),
    Action(ActionError),
    Net(NetErrors),
    Struct(StructError),
}

#[derive(Debug)]
pub struct VMError {
    pub error_type: VMErrorType,
    pub message: String,
    pub semantic_message: String,
}

pub fn throw(error_type: VMErrorType, vm: &Vm) -> VMError {
    let error = match &error_type {
        VMErrorType::TypeCoercionError(v) => {
            let source = if let Some(origin) = &v.origin {
                origin
            } else {
                &v.value.to_string(vm)
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
        VMErrorType::TypeError(ai) => match ai {
            TypeError::InvalidArgsCount { expected, received } => (
                "Invalid args count".to_string(),
                format!("expected {}, received {}", expected, received),
            ),
        },
        VMErrorType::InvalidBinaryOperation(v) => (
            "Invalid binary operation".to_string(),
            format!("{} {} {}", v.left.as_str(), v.operator, v.right.as_str()),
        ),
        VMErrorType::DivisionByZero(v) => {
            let source = if let Some(origin) = &v.origin {
                origin
            } else {
                &v.value.to_string(vm)
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
            FsError::WriteError(s) => ("Write error".to_string(), format!("{}", s)),
            FsError::DeleteError(s) => ("Delete error".to_string(), format!("{}", s)),
        },
        VMErrorType::Os(os) => match os {
            OsError::__placeholder(s) => ("dev note: implement this".to_string(), s.clone()),
        },
        VMErrorType::AI(ai) => match ai {
            AIError::AIFetchError(s) => ("AI fetch error".to_string(), format!("{}", s)),
            AIError::AIEngineNotSet() => (
                "AI engine not set".to_string(),
                format!("set SELF_AI_ENGINE envar to an engine"),
            ),
            AIError::AIEngineNotImplemented(s) => {
                ("AI engine not implemented".to_string(), format!("{}", s))
            }
        },
        VMErrorType::Action(a) => match a {
            ActionError::InvalidModule(s) => (
                "Invalid module".to_string(),
                format!("module '{}' does not exist on self's stdlib", s),
            ),
            ActionError::InvalidMember { module, member } => (
                "Invalid member".to_string(),
                format!("member '{}' does not exist on '{}' module", member, module),
            ),
        },
        VMErrorType::Net(net) => match net {
            NetErrors::NetConnectError(s) => {
                ("Network connection error".to_string(), format!("{}", s))
            }
            NetErrors::WriteError(s) => (
                "Socket write error".to_string(),
                format!("couldn't write to {}", s),
            ),
            NetErrors::ReadError(s) => (
                "Socket write error".to_string(),
                format!("couldn't read from {}", s),
            ),
        },
        VMErrorType::Struct(strc) => match strc {
            StructError::FieldNotFound { field, struct_type } => (
                "Field not found".to_string(),
                format!("'{}' on {}", field, struct_type),
            ),
        },
    };

    VMError {
        error_type: error_type,
        message: error.0,
        semantic_message: error.1,
    }
}

#[derive(Debug)]
pub struct InvalidBinaryOperation {
    pub left: DataType,
    pub right: DataType,
    pub operator: String,
}
