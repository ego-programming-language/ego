use crate::types::{raw::RawValue, Value};

pub fn print_handler(args: Vec<String>, debug: bool, newline_end: bool) {
    for arg in args {
        if debug {
            println!("PRINTLN -> {arg}");
        } else {
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
