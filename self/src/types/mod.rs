pub mod bool;
pub mod f64;
pub mod i32;
pub mod i64;
pub mod u32;
pub mod u64;
pub mod utf8;

use bool::Bool;
use f64::F64;
use i32::I32;
use i64::I64;
use u32::U32;
use u64::U64;
use utf8::Utf8;

use super::opcodes::DataType;

#[derive(Debug, Clone)]
pub enum Value {
    I32(I32),
    I64(I64),
    U32(U32),
    U64(U64),
    F64(F64),
    Utf8(Utf8),
    Bool(Bool),
    Nothing,
}

impl Value {
    pub fn get_type(&self) -> DataType {
        match self {
            Value::I32(_) => DataType::I32,
            Value::I64(_) => DataType::I64,
            Value::U32(_) => DataType::U32,
            Value::U64(_) => DataType::U64,
            Value::F64(_) => DataType::F64,
            Value::Utf8(_) => DataType::Utf8,
            Value::Bool(_) => DataType::Bool,
            Value::Nothing => DataType::Nothing,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::I32(x) => x.value.to_string(),
            Value::I64(x) => x.value.to_string(),
            Value::U32(x) => x.value.to_string(),
            Value::U64(x) => x.value.to_string(),
            Value::F64(x) => x.value.to_string(),
            Value::Utf8(x) => x.value.to_string(),
            Value::Bool(x) => x.value.to_string(),
            Value::Nothing => "nothing".to_string(),
        }
    }

    pub fn as_isize(&self) -> Option<isize> {
        match self {
            Value::I32(x) => Some(x.value as isize),
            Value::I64(x) => Some(x.value as isize),
            Value::U32(_) => None,
            Value::U64(_) => None,
            Value::F64(_) => None,
            Value::Utf8(_) => None,
            Value::Bool(_) => None,
            Value::Nothing => None,
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        match self {
            Value::I32(_) => None,
            Value::I64(_) => None,
            Value::U32(x) => Some(x.value as usize),
            Value::U64(x) => Some(x.value as usize),
            Value::F64(_) => None,
            Value::Utf8(_) => None,
            Value::Bool(_) => None,
            Value::Nothing => None,
        }
    }
}
