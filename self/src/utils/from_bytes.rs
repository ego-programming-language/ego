use crate::{
    opcodes::DataType,
    types::raw::{
        bool::Bool, f64::F64, i32::I32, i64::I64, u32::U32, u64::U64, utf8::Utf8, RawValue,
    },
};

pub fn bytes_to_data(data_type: &DataType, value: &Vec<u8>) -> (RawValue, String) {
    let printable_value;
    let value = match data_type {
        DataType::I32 => {
            let value = i32::from_le_bytes(
                value
                    .as_slice()
                    .try_into()
                    .expect("Provided value is incorrect"),
            );
            printable_value = value.to_string();
            RawValue::I32(I32::new(value))
        }
        DataType::I64 => {
            let value = i64::from_le_bytes(
                value
                    .as_slice()
                    .try_into()
                    .expect("Provided value is incorrect"),
            );
            printable_value = value.to_string();
            RawValue::I64(I64::new(value))
        }
        DataType::U32 => {
            let value = u32::from_le_bytes(
                value
                    .as_slice()
                    .try_into()
                    .expect("Provided value is incorrect"),
            );
            printable_value = value.to_string();
            RawValue::U32(U32::new(value))
        }
        DataType::U64 => {
            let value = u64::from_le_bytes(
                value
                    .as_slice()
                    .try_into()
                    .expect("Provided value is incorrect"),
            );
            printable_value = value.to_string();
            RawValue::U64(U64::new(value))
        }
        DataType::F64 => {
            let value = f64::from_le_bytes(
                value
                    .as_slice()
                    .try_into()
                    .expect("Provided value is incorrect"),
            );
            printable_value = value.to_string();
            RawValue::F64(F64::new(value))
        }
        DataType::Utf8 => {
            let value =
                String::from_utf8(value.clone()).expect("Provided value is not valid UTF-8");
            printable_value = value.to_string();
            RawValue::Utf8(Utf8::new(value))
        }
        DataType::Bool => {
            if value.len() > 1 {
                panic!("Bad boolean value")
            }

            let value = if value[0] == 0x00 {
                printable_value = "false".to_string();
                false
            } else {
                printable_value = "true".to_string();
                true
            };
            RawValue::Bool(Bool::new(value))
        }
        DataType::Nothing => {
            printable_value = "nothing".to_string();
            RawValue::Nothing
        }
        _ => {
            panic!("Unsupported type to get data from")
        }
    };

    (value, printable_value)
}
