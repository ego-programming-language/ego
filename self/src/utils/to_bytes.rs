use super::Number;

pub fn bytes_from_64(num: Number) -> [u8; 8] {
    match num {
        Number::U64(v) => v.to_le_bytes(),
        Number::I64(v) => v.to_le_bytes(),
        _ => {
            println!("Bad type to get bytes from");
            std::process::exit(1);
        }
    }
}

pub fn bytes_from_32(num: Number) -> [u8; 4] {
    match num {
        Number::U32(v) => v.to_le_bytes(),
        Number::I32(v) => v.to_le_bytes(),
        _ => {
            println!("Bad type to get bytes from");
            std::process::exit(1);
        }
    }
}
