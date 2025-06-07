use super::Number;

pub fn bytes_from_float(num: Number) -> [u8; 8] {
    match num {
        Number::F64(v) => v.to_le_bytes(),
        _ => {
            println!("Bad type to get bytes from");
            std::process::exit(1);
        }
    }
}

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

pub fn bytes_from_utf8(string: &String) -> [u8; 8] {
    let mut buffer = [0u8; 8];
    let bytes = string.as_bytes();

    let len = bytes.len().min(8);
    buffer[..len].copy_from_slice(&bytes[..len]);

    buffer
}
