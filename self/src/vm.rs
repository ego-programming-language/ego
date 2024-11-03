use crate::translator::Translator;
use crate::utils::from_bytes::bytes_to_data;

use super::instructions::*;
use super::symbol_table::*;
use super::types::*;

use self::i32::I32;
use self::i64::I64;
use self::u32::U32;
use self::u64::U64;

pub struct Vm {
    operand_stack: Vec<Value>,
    symbol_table: SymbolTable,
    instructions: Vec<Instruction>,
    pc: usize,
}

impl Vm {
    pub fn new(bytecode: Vec<u8>) -> Vm {
        let mut translator = Translator::new(bytecode);
        let instructions = translator.translate();

        Vm {
            operand_stack: vec![],
            symbol_table: SymbolTable::new(),
            instructions,
            pc: 0,
        }
    }

    pub fn run(&mut self, args: &Vec<String>) {
        let debug = args.contains(&"-d".to_string());
        while self.pc < self.instructions.len() {
            let instruction = &self.instructions[self.pc];
            match &instruction {
                Instruction::LoadConst { data_type, value } => {
                    let (value, printable_value) = bytes_to_data(data_type, value);
                    self.operand_stack.push(value);
                    if debug {
                        println!("LOAD_CONST <- {:?}({printable_value})", data_type);
                    }
                }
                Instruction::StoreVar {
                    data_type,
                    value,
                    mutable,
                } => {
                    let (value, printable_value) = bytes_to_data(data_type, value);
                    // todo: self.symbol_table.add_key_value("", value);
                    if debug {
                        println!(
                            "STORE_VAR: {} {:?}({})",
                            if *mutable { "mut" } else { "inmutable" },
                            data_type,
                            printable_value
                        );
                    }
                }
                Instruction::Add => {
                    let right_operand = self.operand_stack.pop();
                    let left_operand = self.operand_stack.pop();

                    if left_operand.is_none() || right_operand.is_none() {
                        panic!("Operands stack underflow");
                    };

                    let operands = (left_operand.unwrap(), right_operand.unwrap());
                    let operands_types = (operands.0.get_type(), operands.1.get_type());

                    if operands_types.0 != operands_types.1 {
                        panic!("No explicit coercion. Operands type mismatch.");
                    }

                    match operands {
                        (Value::I32(l), Value::I32(r)) => {
                            self.operand_stack
                                .push(Value::I32(I32::new(l.value + r.value)));
                            if debug {
                                println!("ADD -> {:?}", l.value + r.value);
                            }
                        }
                        (Value::I64(l), Value::I64(r)) => {
                            self.operand_stack
                                .push(Value::I64(I64::new(l.value + r.value)));
                            if debug {
                                println!("ADD -> {:?}", l.value + r.value);
                            }
                        }
                        (Value::U32(l), Value::U32(r)) => {
                            self.operand_stack
                                .push(Value::U32(U32::new(l.value + r.value)));
                            if debug {
                                println!("ADD -> {:?}", l.value + r.value);
                            }
                        }
                        (Value::Nothing, Value::Nothing) => {
                            self.operand_stack.push(Value::Nothing);
                            if debug {
                                println!("ADD -> nothing");
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::Print { number_of_args } => {
                    let mut counter = 0;
                    let mut args = vec![];
                    while &counter < number_of_args {
                        if let Some(v) = self.operand_stack.pop() {
                            args.push(v);
                        } else {
                            panic!("Cannot get arg to print")
                        }
                        counter += 1;
                    }

                    args.reverse();
                    for arg in args {
                        if debug {
                            match arg {
                                Value::I32(x) => println!("PRINT -> {}", x.value),
                                Value::I64(x) => println!("PRINT -> {}", x.value),
                                Value::U32(x) => println!("PRINT -> {}", x.value),
                                Value::U64(x) => println!("PRINT -> {}", x.value),
                                Value::Utf8(x) => println!("PRINT -> {}", x.value),
                                Value::Bool(x) => println!("PRINT -> {}", x.value),
                                Value::Nothing => println!("PRINT -> nothing"),
                                // Handle other types as necessary
                            }
                        } else {
                            print!("{}", arg.to_string());
                        }
                    }
                }
                _ => {
                    panic!("Unhandled instruction")
                }
            }

            self.pc += 1; // increment program counter
        }
    }

    pub fn debug_bytecode(&mut self) {
        println!("\n--- BYTECODE INSTRUCTIONS ----------\n");
        println!("{:#?}", self.instructions)
    }
}
