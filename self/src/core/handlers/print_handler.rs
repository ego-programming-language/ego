use crate::types::{raw::RawValue, Value};

pub fn print_handler(args: Vec<Value>, debug: bool, newline_end: bool) {
    for arg in args {
        if debug {
            match arg {
                Value::RawValue(x) => match x {
                    RawValue::I32(x) => println!("PRINTLN -> {}", x.value),
                    RawValue::I64(x) => println!("PRINTLN -> {}", x.value),
                    RawValue::U32(x) => println!("PRINTLN -> {}", x.value),
                    RawValue::U64(x) => println!("PRINTLN -> {}", x.value),
                    RawValue::F64(x) => println!("PRINTLN -> {}", x.value),
                    RawValue::Utf8(x) => println!("PRINTLN -> {}", x.value),
                    RawValue::Bool(x) => println!("PRINTLN -> {}", x.value),
                    RawValue::Nothing => println!("PRINTLN -> nothing"),
                },
                Value::HeapRef(x) => {
                    println!("PRINTLN -> {}", x.get_address())
                }
            }
        } else {
            let arg = arg.to_string();
            let mut iter = arg.split("\\n").enumerate().peekable();

            while let Some((_index, string)) = iter.next() {
                if iter.peek().is_none() {
                    print!("{}", string);
                } else {
                    println!("{}", string);
                }
            }
        }
    }

    if newline_end {
        print!("\n")
    };
}
