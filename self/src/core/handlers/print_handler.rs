use crate::types::Value;

pub fn print_handler(args: Vec<Value>, debug: bool, newline_end: bool) {
    for arg in args {
        if debug {
            match arg {
                Value::I32(x) => println!("PRINTLN -> {}", x.value),
                Value::I64(x) => println!("PRINTLN -> {}", x.value),
                Value::U32(x) => println!("PRINTLN -> {}", x.value),
                Value::U64(x) => println!("PRINTLN -> {}", x.value),
                Value::F64(x) => println!("PRINTLN -> {}", x.value),
                Value::Utf8(x) => println!("PRINTLN -> {}", x.value),
                Value::Bool(x) => println!("PRINTLN -> {}", x.value),
                Value::Nothing => println!("PRINTLN -> nothing"),
                // Handle other types as necessary
            }
        } else {
            let line_end = if newline_end { "\n" } else { "" };
            let arg = arg.to_string() + line_end;
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
}
