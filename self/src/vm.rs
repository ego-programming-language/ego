use crate::core::handlers::call_handler;
use crate::core::handlers::call_handler::call_handler;
use crate::core::handlers::foreign_handlers::ForeignHandlers;
use crate::opcodes::DataType;
use crate::translator::Translator;
use crate::utils::foreign_handlers_utils::get_foreign_handlers;
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
    handlers: ForeignHandlers,
}

impl Vm {
    pub fn new(bytecode: Vec<u8>) -> Vm {
        let mut translator = Translator::new(bytecode);
        let instructions = translator.translate();
        let mut handlers = ForeignHandlers::new();
        let foreign_handlers = get_foreign_handlers();

        if let Some(loaded_handlers) = foreign_handlers {
            for handler in loaded_handlers.functions {
                handlers.add(handler);
            }
        }

        Vm {
            operand_stack: vec![],
            symbol_table: SymbolTable::new(),
            instructions,
            pc: 0,
            handlers,
        }
    }

    pub fn run(&mut self, args: &Vec<String>) {
        let debug = args.contains(&"-d".to_string());
        while self.pc < self.instructions.len() {
            let instruction = self.instructions[self.pc].clone();
            match &instruction {
                Instruction::LoadConst { data_type, value } => {
                    let (value, printable_value) = bytes_to_data(data_type, value);
                    self.operand_stack.push(value);
                    if debug {
                        println!("LOAD_CONST <- {:?}({printable_value})", data_type);
                    }
                }
                Instruction::LoadVar {
                    data_type,
                    identifier,
                } => {
                    let (identifier_name, printable_value) = bytes_to_data(data_type, identifier);

                    if let Value::Utf8(v) = identifier_name {
                        let identifier_value = self.symbol_table.get_value(v.value);
                        if let Some(v) = identifier_value {
                            self.operand_stack.push(v);
                            if debug {
                                println!("LOAD_VAR <- {:?}({printable_value})", data_type);
                            }
                        } else {
                            // should be handled with ego errors
                            panic!("{printable_value} is not defined")
                        }
                    } else {
                        panic!("LOAD_VAR identifier should be a string")
                    }
                }
                Instruction::StoreVar {
                    identifier,
                    mutable,
                } => {
                    let stack_stored_value = self.operand_stack.pop();
                    if let Some(v) = stack_stored_value {
                        let datatype = v.get_type();
                        let printable_value = v.to_string();
                        self.symbol_table.add_key_value(identifier.clone(), v);
                        if debug {
                            println!(
                                "STORE_VAR[{}] <- {:?}({}) as {}",
                                if *mutable { "MUT" } else { "INMUT" },
                                datatype,
                                printable_value,
                                identifier,
                            );
                        }
                    } else {
                        // todo: use self-vm errors
                        panic!("STACK UNDERFLOW")
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
                    let args = self.get_stack_values(number_of_args);
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
                            // print with newlines
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
                }
                Instruction::Call { number_of_args } => {
                    let args = self.get_stack_values(number_of_args);

                    if let DataType::Utf8 = args[0].get_type() {
                        if debug {
                            println!("CALL -> {}", args[0].to_string())
                        }
                        call_handler(&self.handlers, args)
                    } else {
                        panic!("Call first argument must be a string")
                    }
                }
                _ => {
                    panic!("Unhandled instruction")
                }
            }

            self.pc += 1; // increment program counter
        }
    }

    pub fn get_stack_values(&mut self, num_of_values: &u32) -> Vec<Value> {
        let mut counter = 0;
        let mut args = vec![];
        while &counter < num_of_values {
            if let Some(v) = self.operand_stack.pop() {
                args.push(v);
            } else {
                panic!("Cannot get arg of call function")
            }
            counter += 1;
        }

        args.reverse();
        args
    }

    pub fn debug_bytecode(&mut self) {
        println!("\n--- BYTECODE INSTRUCTIONS ----------\n");
        println!("{:#?}", self.instructions)
    }
}
