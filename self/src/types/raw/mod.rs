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

use crate::opcodes::DataType;

#[derive(Debug, Clone)]
pub enum RawValue {
    I32(I32),
    I64(I64),
    U32(U32),
    U64(U64),
    F64(F64),
    Utf8(Utf8),
    Bool(Bool),
    Nothing,
}

impl RawValue {
    pub fn get_type(&self) -> DataType {
        match self {
            RawValue::I32(_) => DataType::I32,
            RawValue::I64(_) => DataType::I64,
            RawValue::U32(_) => DataType::U32,
            RawValue::U64(_) => DataType::U64,
            RawValue::F64(_) => DataType::F64,
            RawValue::Utf8(_) => DataType::Utf8,
            RawValue::Bool(_) => DataType::Bool,
            RawValue::Nothing => DataType::Nothing,
        }
    }

    pub fn get_type_string(&self) -> String {
        match self {
            RawValue::I32(_) => "I32".to_string(),
            RawValue::I64(_) => "I64".to_string(),
            RawValue::U32(_) => "U32".to_string(),
            RawValue::U64(_) => "U64".to_string(),
            RawValue::F64(_) => "F64".to_string(),
            RawValue::Utf8(_) => "UTF8".to_string(),
            RawValue::Bool(_) => "BOOL".to_string(),
            RawValue::Nothing => "NOTHING".to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            RawValue::I32(x) => x.value.to_string(),
            RawValue::I64(x) => x.value.to_string(),
            RawValue::U32(x) => x.value.to_string(),
            RawValue::U64(x) => x.value.to_string(),
            RawValue::F64(x) => x.value.to_string(),
            RawValue::Utf8(x) => x.value.to_string(),
            RawValue::Bool(x) => x.value.to_string(),
            RawValue::Nothing => "nothing".to_string(),
        }
    }

    pub fn as_isize(&self) -> Option<isize> {
        match self {
            RawValue::I32(x) => Some(x.value as isize),
            RawValue::I64(x) => Some(x.value as isize),
            RawValue::U32(_) => None,
            RawValue::U64(_) => None,
            RawValue::F64(_) => None,
            RawValue::Utf8(_) => None,
            RawValue::Bool(_) => None,
            RawValue::Nothing => None,
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        match self {
            RawValue::I32(_) => None,
            RawValue::I64(_) => None,
            RawValue::U32(x) => Some(x.value as usize),
            RawValue::U64(x) => Some(x.value as usize),
            RawValue::F64(_) => None,
            RawValue::Utf8(_) => None,
            RawValue::Bool(_) => None,
            RawValue::Nothing => None,
        }
    }
}
