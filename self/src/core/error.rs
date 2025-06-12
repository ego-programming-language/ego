pub enum VMErrorType {
    TypeCoercionError,
}

pub struct VMError {
    pub error_type: VMErrorType,
    pub message: String,
    pub semantic_message: String,
}

pub fn throw(error_type: VMErrorType, error_message: String) -> VMError {
    let error_type_string = match error_type {
        VMErrorType::TypeCoercionError => "Type coercion error".to_string(),
    };

    VMError {
        error_type: error_type,
        message: error_type_string,
        semantic_message: error_message,
    }
}
