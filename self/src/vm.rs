use crate::core::error::InvalidBinaryOperation;
use crate::core::error::VMErrorType;
use crate::core::execution::VMExecutionResult;
use crate::core::handlers::call_handler::call_handler;
use crate::core::handlers::foreign_handlers::ForeignHandlers;
use crate::core::handlers::print_handler::print_handler;
use crate::opcodes::DataType;
use crate::translator::Translator;
use crate::types::bool::Bool;
use crate::types::f64::F64;
use crate::types::u64::U64;
use crate::types::utf8::Utf8;
use crate::utils::foreign_handlers_utils::get_foreign_handlers;
use crate::utils::from_bytes::bytes_to_data;

use super::instructions::*;
use super::symbol_table::*;
use super::types::*;

use self::i32::I32;
use self::i64::I64;
use self::u32::U32;

pub struct Vm {
    operand_stack: Vec<StackValue>,
    symbol_table: SymbolTable,
    instructions: Vec<Instruction>,
    pc: usize,
    handlers: ForeignHandlers,
}

pub struct StackValue {
    pub value: Value,
    pub origin: Option<String>,
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

    pub fn run(&mut self, args: &Vec<String>) -> VMExecutionResult {
        let debug = args.contains(&"-d".to_string());
        while self.pc < self.instructions.len() {
            let instruction = self.instructions[self.pc].clone();
            match &instruction {
                Instruction::LoadConst { data_type, value } => {
                    let (value, printable_value) = bytes_to_data(data_type, value);
                    self.push_to_stack(value, None);
                    if debug {
                        println!("LOAD_CONST <- {:?}({printable_value})", data_type);
                    }
                }
                Instruction::LoadVar {
                    data_type,
                    identifier,
                } => {
                    let (identifier_name, printable_value) = bytes_to_data(data_type, identifier);

                    if let Value::Utf8(i) = identifier_name {
                        let identifier_value = self.symbol_table.get_value(&i.value);
                        if let Some(v) = identifier_value {
                            self.push_to_stack(v, Some(i.value));
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
                        let datatype = v.value.get_type();
                        let printable_value = v.value.to_string();
                        self.symbol_table.add_key_value(identifier.clone(), v.value);
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
                    let operands_values = (&operands.0.value, &operands.1.value);
                    let operands_types =
                        (operands_values.0.get_type(), operands_values.1.get_type());

                    if operands_types.0 != operands_types.1 {
                        return VMExecutionResult::terminate_with_errors(
                            VMErrorType::TypeCoercionError(operands.1),
                        );
                    }

                    match operands_values {
                        (Value::I32(l), Value::I32(r)) => {
                            self.push_to_stack(Value::I32(I32::new(l.value + r.value)), None);
                            if debug {
                                println!("ADD -> {:?}", l.value + r.value);
                            }
                        }
                        (Value::I64(l), Value::I64(r)) => {
                            self.push_to_stack(Value::I64(I64::new(l.value + r.value)), None);
                            if debug {
                                println!("ADD -> {:?}", l.value + r.value);
                            }
                        }
                        (Value::U32(l), Value::U32(r)) => {
                            self.push_to_stack(Value::U32(U32::new(l.value + r.value)), None);
                            if debug {
                                println!("ADD -> {:?}", l.value + r.value);
                            }
                        }
                        (Value::U64(l), Value::U64(r)) => {
                            self.push_to_stack(Value::U64(U64::new(l.value + r.value)), None);
                            if debug {
                                println!("ADD -> {:?}", l.value + r.value);
                            }
                        }
                        (Value::F64(l), Value::F64(r)) => {
                            self.push_to_stack(Value::F64(F64::new(l.value + r.value)), None);
                            if debug {
                                println!("ADD -> {:?}", l.value + r.value);
                            }
                        }
                        (Value::Nothing, Value::Nothing) => {
                            self.push_to_stack(Value::Nothing, None);
                            if debug {
                                println!("ADD -> nothing");
                            }
                        }
                        (Value::Utf8(l), Value::Utf8(r)) => {
                            self.push_to_stack(
                                Value::Utf8(Utf8::new(l.value.to_string() + r.value.as_str())),
                                None,
                            );
                            if debug {
                                println!("ADD -> {:?}", l.value.to_string() + r.value.as_str());
                            }
                        }
                        (Value::Bool(l), Value::Bool(r)) => {
                            let result = l.value || r.value;
                            self.push_to_stack(Value::Bool(Bool::new(result)), None);
                            if debug {
                                println!("ADD -> {:?}", result);
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::Substract => {
                    let right_operand = self.operand_stack.pop();
                    let left_operand = self.operand_stack.pop();

                    if left_operand.is_none() || right_operand.is_none() {
                        panic!("Operands stack underflow");
                    };

                    let operands = (left_operand.unwrap(), right_operand.unwrap());
                    let operands_values = (&operands.0.value, &operands.1.value);
                    let operands_types =
                        (operands_values.0.get_type(), operands_values.1.get_type());

                    if operands_types.0 != operands_types.1 {
                        return VMExecutionResult::terminate_with_errors(
                            VMErrorType::TypeCoercionError(operands.1),
                        );
                    }

                    match operands_values {
                        (Value::I32(l), Value::I32(r)) => {
                            self.push_to_stack(Value::I32(I32::new(l.value - r.value)), None);
                            if debug {
                                println!("SUBSTRACT -> {:?}", l.value - r.value);
                            }
                        }
                        (Value::I64(l), Value::I64(r)) => {
                            self.push_to_stack(Value::I64(I64::new(l.value - r.value)), None);
                            if debug {
                                println!("SUBSTRACT -> {:?}", l.value - r.value);
                            }
                        }
                        (Value::U32(l), Value::U32(r)) => {
                            self.push_to_stack(Value::U32(U32::new(l.value - r.value)), None);
                            if debug {
                                println!("SUBSTRACT -> {:?}", l.value - r.value);
                            }
                        }
                        (Value::U64(l), Value::U64(r)) => {
                            self.push_to_stack(Value::U64(U64::new(l.value - r.value)), None);
                            if debug {
                                println!("SUBSTRACT -> {:?}", l.value - r.value);
                            }
                        }
                        (Value::F64(l), Value::F64(r)) => {
                            self.push_to_stack(Value::F64(F64::new(l.value - r.value)), None);
                            if debug {
                                println!("SUBSTRACT -> {:?}", l.value - r.value);
                            }
                        }
                        (Value::Nothing, Value::Nothing) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Nothing,
                                    right: DataType::Nothing,
                                    operator: "-".to_string(),
                                }),
                            );
                        }
                        (Value::Utf8(_), Value::Utf8(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Utf8,
                                    right: DataType::Utf8,
                                    operator: "-".to_string(),
                                }),
                            );
                        }
                        (Value::Bool(_), Value::Bool(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Bool,
                                    right: DataType::Bool,
                                    operator: "-".to_string(),
                                }),
                            );
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::Multiply => {
                    let right_operand = self.operand_stack.pop();
                    let left_operand = self.operand_stack.pop();

                    if left_operand.is_none() || right_operand.is_none() {
                        panic!("Operands stack underflow");
                    };

                    let operands = (left_operand.unwrap(), right_operand.unwrap());
                    let operands_values = (&operands.0.value, &operands.1.value);
                    let operands_types =
                        (operands_values.0.get_type(), operands_values.1.get_type());

                    if operands_types.0 != operands_types.1 {
                        return VMExecutionResult::terminate_with_errors(
                            VMErrorType::TypeCoercionError(operands.1),
                        );
                    }

                    match operands_values {
                        (Value::I32(l), Value::I32(r)) => {
                            self.push_to_stack(Value::I32(I32::new(l.value * r.value)), None);
                            if debug {
                                println!("MULTIPLY -> {:?}", l.value * r.value);
                            }
                        }
                        (Value::I64(l), Value::I64(r)) => {
                            self.push_to_stack(Value::I64(I64::new(l.value * r.value)), None);
                            if debug {
                                println!("MULTIPLY -> {:?}", l.value * r.value);
                            }
                        }
                        (Value::U32(l), Value::U32(r)) => {
                            self.push_to_stack(Value::U32(U32::new(l.value * r.value)), None);
                            if debug {
                                println!("MULTIPLY -> {:?}", l.value * r.value);
                            }
                        }
                        (Value::U64(l), Value::U64(r)) => {
                            self.push_to_stack(Value::U64(U64::new(l.value * r.value)), None);
                            if debug {
                                println!("MULTIPLY -> {:?}", l.value * r.value);
                            }
                        }
                        (Value::F64(l), Value::F64(r)) => {
                            self.push_to_stack(Value::F64(F64::new(l.value * r.value)), None);
                            if debug {
                                println!("MULTIPLY -> {:?}", l.value * r.value);
                            }
                        }
                        (Value::Nothing, Value::Nothing) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Nothing,
                                    right: DataType::Nothing,
                                    operator: "*".to_string(),
                                }),
                            );
                        }
                        (Value::Utf8(_), Value::Utf8(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Utf8,
                                    right: DataType::Utf8,
                                    operator: "*".to_string(),
                                }),
                            );
                        }
                        (Value::Bool(_), Value::Bool(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Bool,
                                    right: DataType::Bool,
                                    operator: "*".to_string(),
                                }),
                            );
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::Divide => {
                    let right_operand = self.operand_stack.pop();
                    let left_operand = self.operand_stack.pop();

                    if left_operand.is_none() || right_operand.is_none() {
                        panic!("Operands stack underflow");
                    };

                    let operands = (left_operand.unwrap(), right_operand.unwrap());
                    let operands_values = (&operands.0.value, &operands.1.value);
                    let operands_types =
                        (operands_values.0.get_type(), operands_values.1.get_type());

                    if operands_types.0 != operands_types.1 {
                        return VMExecutionResult::terminate_with_errors(
                            VMErrorType::TypeCoercionError(operands.1),
                        );
                    }

                    match operands_values {
                        (Value::I32(l), Value::I32(r)) => {
                            if r.value == 0 {
                                return VMExecutionResult::terminate_with_errors(
                                    VMErrorType::DivisionByZero(operands.0),
                                );
                            }
                            self.push_to_stack(Value::I32(I32::new(l.value / r.value)), None);
                            if debug {
                                println!("DIVIDE -> {:?}", l.value / r.value);
                            }
                        }
                        (Value::I64(l), Value::I64(r)) => {
                            if r.value == 0 {
                                return VMExecutionResult::terminate_with_errors(
                                    VMErrorType::DivisionByZero(operands.0),
                                );
                            }
                            self.push_to_stack(Value::I64(I64::new(l.value / r.value)), None);
                            if debug {
                                println!("DIVIDE -> {:?}", l.value / r.value);
                            }
                        }
                        (Value::U32(l), Value::U32(r)) => {
                            if r.value == 0 {
                                return VMExecutionResult::terminate_with_errors(
                                    VMErrorType::DivisionByZero(operands.0),
                                );
                            }
                            self.push_to_stack(Value::U32(U32::new(l.value / r.value)), None);
                            if debug {
                                println!("DIVIDE -> {:?}", l.value / r.value);
                            }
                        }
                        (Value::U64(l), Value::U64(r)) => {
                            if r.value == 0 {
                                return VMExecutionResult::terminate_with_errors(
                                    VMErrorType::DivisionByZero(operands.0),
                                );
                            }
                            self.push_to_stack(Value::U64(U64::new(l.value / r.value)), None);
                            if debug {
                                println!("DIVIDE -> {:?}", l.value / r.value);
                            }
                        }
                        (Value::F64(l), Value::F64(r)) => {
                            if r.value == 0.0 {
                                return VMExecutionResult::terminate_with_errors(
                                    VMErrorType::DivisionByZero(operands.0),
                                );
                            }
                            self.push_to_stack(Value::F64(F64::new(l.value / r.value)), None);
                            if debug {
                                println!("DIVIDE -> {:?}", l.value / r.value);
                            }
                        }
                        (Value::Nothing, Value::Nothing) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Nothing,
                                    right: DataType::Nothing,
                                    operator: "/".to_string(),
                                }),
                            );
                        }
                        (Value::Utf8(_), Value::Utf8(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Utf8,
                                    right: DataType::Utf8,
                                    operator: "/".to_string(),
                                }),
                            );
                        }
                        (Value::Bool(_), Value::Bool(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Bool,
                                    right: DataType::Bool,
                                    operator: "/".to_string(),
                                }),
                            );
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::Print { number_of_args } => {
                    let args = self.get_stack_values(number_of_args);
                    print_handler(args, debug, false)
                }
                Instruction::Println { number_of_args } => {
                    let args = self.get_stack_values(number_of_args);
                    print_handler(args, debug, true)
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

        VMExecutionResult::terminate()
    }

    pub fn get_stack_values(&mut self, num_of_values: &u32) -> Vec<Value> {
        let mut args = Vec::with_capacity(*num_of_values as usize);

        for _ in 0..*num_of_values {
            match self.operand_stack.pop() {
                Some(v) => args.push(v.value),
                None => panic!("Cannot get argument for function call: stack underflow"),
            }
        }

        args.reverse(); // invocation order
        args
    }

    pub fn push_to_stack(&mut self, value: Value, origin: Option<String>) {
        self.operand_stack.push(StackValue { value, origin });
    }

    pub fn debug_bytecode(&mut self) {
        println!("\n--- BYTECODE INSTRUCTIONS ----------\n");
        println!("{:#?}", self.instructions)
    }
}
