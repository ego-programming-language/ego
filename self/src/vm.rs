use crate::core::error::InvalidBinaryOperation;
use crate::core::error::VMErrorType;
use crate::core::execution::VMExecutionResult;
use crate::core::handlers::call_handler::call_handler;
use crate::core::handlers::foreign_handlers::ForeignHandlers;
use crate::core::handlers::print_handler::print_handler;
use crate::opcodes::DataType;
use crate::opcodes::Opcode;
use crate::translator::Translator;
use crate::types::raw::Value;
use crate::types::raw::{bool::Bool, f64::F64, i32::I32, i64::I64, u32::U32, u64::U64, utf8::Utf8};
use crate::utils::foreign_handlers_utils::get_foreign_handlers;
use crate::utils::from_bytes::bytes_to_data;

use super::symbol_table::*;
use super::types::*;

pub struct Vm {
    operand_stack: Vec<StackValue>,
    symbol_table: SymbolTable,
    bytecode: Vec<u8>,
    pc: usize,
    handlers: ForeignHandlers,
}

#[derive(Debug)]
pub struct StackValue {
    pub value: Value,
    pub origin: Option<String>,
}

impl Vm {
    pub fn new(bytecode: Vec<u8>) -> Vm {
        //let mut translator = Translator::new(bytecode);
        //let instructions = translator.translate();
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
            bytecode,
            pc: 0,
            handlers,
        }
    }

    pub fn run(&mut self, args: &Vec<String>) -> VMExecutionResult {
        let debug = args.contains(&"-d".to_string());
        if debug {
            println!("last PC value: {}", self.bytecode.len());
            println!("-");
        }
        while self.pc < self.bytecode.len() {
            match Opcode::to_opcode(self.bytecode[self.pc]) {
                Opcode::LoadConst => {
                    // parsing
                    if self.pc + 1 >= self.bytecode.len() {
                        panic!("Invalid LOAD_CONST instruction at position {}", self.pc);
                    }

                    self.pc += 1;
                    let (data_type, value_bytes) = self.get_value_length();

                    // execution
                    let (value, printable_value) = bytes_to_data(&data_type, &value_bytes);
                    self.push_to_stack(value, None);
                    if debug {
                        println!("LOAD_CONST <- {:?}({printable_value})", data_type);
                    }

                    self.pc += 1;
                }
                Opcode::LoadVar => {
                    // parsing
                    if self.pc + 1 >= self.bytecode.len() {
                        panic!("Invalid LOAD_VAR instruction at position {}", self.pc);
                    }

                    self.pc += 1;
                    let (data_type, value_bytes) = self.get_value_length();

                    // execution
                    let (identifier_name, printable_value) =
                        bytes_to_data(&data_type, &value_bytes);

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

                    self.pc += 1;
                }
                Opcode::JumpIfFalse => {
                    let offset = Vm::read_offset(&self.bytecode[self.pc + 1..self.pc + 5]);
                    self.pc += 4;

                    let condition = self.operand_stack.pop();
                    if condition.is_none() {
                        panic!("stack underflow");
                    };

                    let condition = condition.unwrap();
                    match condition.value {
                        Value::Bool(execute_if) => {
                            if debug {
                                println!("JUMP_IF_FALSE <- {:?}({})", execute_if.value, offset);
                            }
                            if !execute_if.value {
                                self.pc += offset as usize;
                            }
                        }
                        _ => {
                            panic!("invalid expression type as condition to jump")
                        }
                    };

                    self.pc += 1;
                }
                Opcode::Jump => {
                    // execution
                    let offset = Vm::read_offset(&self.bytecode[self.pc + 1..self.pc + 5]);
                    self.pc += 4;

                    let target_pc = (self.pc as isize) + offset as isize;
                    if debug {
                        println!("JUMP <- {:?}", target_pc);
                    }
                    self.pc = target_pc as usize;
                }
                Opcode::Print => {
                    // parsing
                    // get u32 value. 4 bytes based on the type plus the current
                    let value_length = 4;
                    if self.pc + value_length >= self.bytecode.len() {
                        panic!("Invalid print instruction at position {}", self.pc);
                    }

                    let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
                    let number_of_args = u32::from_le_bytes(
                        value_bytes.try_into().expect("Provided value is incorrect"),
                    );
                    self.pc += 4;

                    // execution
                    let args = self.get_stack_values(&number_of_args);
                    print_handler(args, debug, false);

                    self.pc += 1;
                }
                Opcode::Println => {
                    // parsing
                    // get u32 value. 4 bytes based on the type plus the current
                    let value_length = 4;
                    if self.pc + value_length >= self.bytecode.len() {
                        panic!("Invalid print instruction at position {}", self.pc);
                    }

                    let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
                    let number_of_args = u32::from_le_bytes(
                        value_bytes.try_into().expect("Provided value is incorrect"),
                    );
                    self.pc += 4;

                    // execution
                    let args = self.get_stack_values(&number_of_args);
                    print_handler(args, debug, true);

                    self.pc += 1;
                }
                Opcode::Add => {
                    // execution
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

                    self.pc += 1;
                }
                Opcode::Substract => {
                    // execution
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

                    self.pc += 1;
                }
                Opcode::Multiply => {
                    // execution
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

                    self.pc += 1;
                }
                Opcode::Divide => {
                    // execution
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

                    self.pc += 1;
                }
                Opcode::GreaterThan => {
                    // execution
                    let right_operand = self.operand_stack.pop();
                    let left_operand = self.operand_stack.pop();

                    if left_operand.is_none() || right_operand.is_none() {
                        panic!("Operands stack underflow");
                    };

                    let operands = (left_operand.unwrap(), right_operand.unwrap());
                    let operands_values = (&operands.0.value, &operands.1.value);

                    match operands_values {
                        (Value::I32(l), Value::I32(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value > r.value)), None);
                            if debug {
                                println!("GREATER_THAN -> {:?}", l.value > r.value);
                            }
                        }
                        (Value::I64(l), Value::I64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value > r.value)), None);
                            if debug {
                                println!("GREATER_THAN -> {:?}", l.value > r.value);
                            }
                        }
                        (Value::U32(l), Value::U32(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value > r.value)), None);
                            if debug {
                                println!("GREATER_THAN -> {:?}", l.value > r.value);
                            }
                        }
                        (Value::U64(l), Value::U64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value > r.value)), None);
                            if debug {
                                println!("GREATER_THAN -> {:?}", l.value > r.value);
                            }
                        }
                        (Value::F64(l), Value::F64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value > r.value)), None);
                            if debug {
                                println!("GREATER_THAN -> {:?}", l.value > r.value);
                            }
                        }
                        (Value::Nothing, Value::Nothing) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Nothing,
                                    right: DataType::Nothing,
                                    operator: ">".to_string(),
                                }),
                            );
                        }
                        (Value::Utf8(_), Value::Utf8(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Utf8,
                                    right: DataType::Utf8,
                                    operator: ">".to_string(),
                                }),
                            );
                        }
                        (Value::Bool(_), Value::Bool(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Bool,
                                    right: DataType::Bool,
                                    operator: ">".to_string(),
                                }),
                            );
                        }
                        _ => unreachable!(),
                    }
                    self.pc += 1;
                }
                Opcode::LessThan => {
                    // execution
                    let right_operand = self.operand_stack.pop();
                    let left_operand = self.operand_stack.pop();

                    if left_operand.is_none() || right_operand.is_none() {
                        panic!("Operands stack underflow");
                    };

                    let operands = (left_operand.unwrap(), right_operand.unwrap());
                    let operands_values = (&operands.0.value, &operands.1.value);

                    match operands_values {
                        (Value::I32(l), Value::I32(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value < r.value)), None);
                            if debug {
                                println!("LESS_THAN -> {:?}", l.value < r.value);
                            }
                        }
                        (Value::I64(l), Value::I64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value < r.value)), None);
                            if debug {
                                println!("LESS_THAN -> {:?}", l.value < r.value);
                            }
                        }
                        (Value::U32(l), Value::U32(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value < r.value)), None);
                            if debug {
                                println!("LESS_THAN -> {:?}", l.value < r.value);
                            }
                        }
                        (Value::U64(l), Value::U64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value < r.value)), None);
                            if debug {
                                println!("LESS_THAN -> {:?}", l.value < r.value);
                            }
                        }
                        (Value::F64(l), Value::F64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value < r.value)), None);
                            if debug {
                                println!("LESS_THAN -> {:?}", l.value < r.value);
                            }
                        }
                        (Value::Nothing, Value::Nothing) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Nothing,
                                    right: DataType::Nothing,
                                    operator: "<".to_string(),
                                }),
                            );
                        }
                        (Value::Utf8(_), Value::Utf8(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Utf8,
                                    right: DataType::Utf8,
                                    operator: "<".to_string(),
                                }),
                            );
                        }
                        (Value::Bool(_), Value::Bool(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Bool,
                                    right: DataType::Bool,
                                    operator: "<".to_string(),
                                }),
                            );
                        }
                        _ => unreachable!(),
                    }
                    self.pc += 1;
                }
                Opcode::Equals => {
                    // execution
                    let right_operand = self.operand_stack.pop();
                    let left_operand = self.operand_stack.pop();

                    if left_operand.is_none() || right_operand.is_none() {
                        panic!("Operands stack underflow");
                    };

                    let operands = (left_operand.unwrap(), right_operand.unwrap());
                    let operands_values = (&operands.0.value, &operands.1.value);

                    match operands_values {
                        (Value::I32(l), Value::I32(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value == r.value)), None);
                            if debug {
                                println!("EQUALS -> {:?}", l.value == r.value);
                            }
                        }
                        (Value::I64(l), Value::I64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value == r.value)), None);
                            if debug {
                                println!("EQUALS -> {:?}", l.value == r.value);
                            }
                        }
                        (Value::U32(l), Value::U32(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value == r.value)), None);
                            if debug {
                                println!("EQUALS -> {:?}", l.value == r.value);
                            }
                        }
                        (Value::U64(l), Value::U64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value == r.value)), None);
                            if debug {
                                println!("EQUALS -> {:?}", l.value == r.value);
                            }
                        }
                        (Value::F64(l), Value::F64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value == r.value)), None);
                            if debug {
                                println!("EQUALS -> {:?}", l.value == r.value);
                            }
                        }
                        (Value::Nothing, Value::Nothing) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Nothing,
                                    right: DataType::Nothing,
                                    operator: "==".to_string(),
                                }),
                            );
                        }
                        (Value::Utf8(_), Value::Utf8(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Utf8,
                                    right: DataType::Utf8,
                                    operator: "==".to_string(),
                                }),
                            );
                        }
                        (Value::Bool(_), Value::Bool(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Bool,
                                    right: DataType::Bool,
                                    operator: "==".to_string(),
                                }),
                            );
                        }
                        _ => unreachable!(),
                    }
                    self.pc += 1;
                }
                Opcode::NotEquals => {
                    // execution
                    let right_operand = self.operand_stack.pop();
                    let left_operand = self.operand_stack.pop();

                    if left_operand.is_none() || right_operand.is_none() {
                        panic!("Operands stack underflow");
                    };

                    let operands = (left_operand.unwrap(), right_operand.unwrap());
                    let operands_values = (&operands.0.value, &operands.1.value);

                    match operands_values {
                        (Value::I32(l), Value::I32(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value != r.value)), None);
                            if debug {
                                println!("NOT_EQUALS -> {:?}", l.value != r.value);
                            }
                        }
                        (Value::I64(l), Value::I64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value != r.value)), None);
                            if debug {
                                println!("NOT_EQUALS -> {:?}", l.value != r.value);
                            }
                        }
                        (Value::U32(l), Value::U32(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value != r.value)), None);
                            if debug {
                                println!("NOT_EQUALS -> {:?}", l.value != r.value);
                            }
                        }
                        (Value::U64(l), Value::U64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value != r.value)), None);
                            if debug {
                                println!("NOT_EQUALS -> {:?}", l.value != r.value);
                            }
                        }
                        (Value::F64(l), Value::F64(r)) => {
                            self.push_to_stack(Value::Bool(Bool::new(l.value != r.value)), None);
                            if debug {
                                println!("NOT_EQUALS -> {:?}", l.value != r.value);
                            }
                        }
                        (Value::Nothing, Value::Nothing) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Nothing,
                                    right: DataType::Nothing,
                                    operator: "!=".to_string(),
                                }),
                            );
                        }
                        (Value::Utf8(_), Value::Utf8(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Utf8,
                                    right: DataType::Utf8,
                                    operator: "!=".to_string(),
                                }),
                            );
                        }
                        (Value::Bool(_), Value::Bool(_)) => {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::InvalidBinaryOperation(InvalidBinaryOperation {
                                    left: DataType::Bool,
                                    right: DataType::Bool,
                                    operator: "!=".to_string(),
                                }),
                            );
                        }
                        _ => unreachable!(),
                    }
                    self.pc += 1;
                }
                Opcode::StoreVar => {
                    // parsing
                    if self.pc + 1 >= self.bytecode.len() {
                        panic!("Invalid STORE_VAR instruction at position {}.", self.pc);
                    } else {
                        self.pc += 1;
                    }

                    // 0x00 inmutable | 0x00 mutable
                    let mutable = match self.bytecode[self.pc] {
                        0x00 => false,
                        0x01 => true,
                        _ => {
                            panic!("Invalid STORE_VAR instruction at position {}. Needed mutability property.", self.pc);
                        }
                    };
                    self.pc += 1;

                    // identifier
                    let (identifier_data_type, identifier_bytes) = self.get_value_length();
                    if identifier_data_type != DataType::Utf8 {
                        panic!("Identifier type should be a string encoded as utf8")
                    }
                    let identifier_name = String::from_utf8(identifier_bytes)
                        .expect("Identifier bytes should be valid UTF-8");

                    // execution
                    let stack_stored_value = self.operand_stack.pop();
                    if let Some(v) = stack_stored_value {
                        let datatype = v.value.get_type();
                        let printable_value = v.value.to_string();
                        self.symbol_table
                            .add_key_value(identifier_name.clone(), v.value);
                        if debug {
                            println!(
                                "STORE_VAR[{}] <- {:?}({}) as {}",
                                if mutable { "MUT" } else { "INMUT" },
                                datatype,
                                printable_value,
                                identifier_name,
                            );
                        }
                    } else {
                        // todo: use self-vm errors
                        panic!("STACK UNDERFLOW")
                    }

                    self.pc += 1;
                }
                Opcode::Call => {
                    // parsing
                    // get u32 value. 4 bytes based on the type plus the current
                    let value_length = 4;
                    if self.pc + value_length >= self.bytecode.len() {
                        panic!("Invalid print instruction at position {}", self.pc);
                    }

                    let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
                    let number_of_args = u32::from_le_bytes(
                        value_bytes.try_into().expect("Provided value is incorrect"),
                    );
                    self.pc += 4;

                    // execution
                    let args = self.get_stack_values(&number_of_args);

                    if let DataType::Utf8 = args[0].get_type() {
                        if debug {
                            println!("CALL -> {}", args[0].to_string())
                        }
                        call_handler(&self.handlers, args)
                    } else {
                        panic!("Call first argument must be a string")
                    }

                    self.pc += 1;
                }
                _ => {
                    println!("unhandled opcode");
                    self.pc += 1;
                }
            };
        }

        VMExecutionResult::terminate()
    }

    fn get_value_length(&mut self) -> (DataType, Vec<u8>) {
        let data_type = DataType::to_opcode(self.bytecode[self.pc]);
        let value_length = match data_type {
            DataType::I32 => 4,
            DataType::I64 => 8,
            DataType::U32 => 4,
            DataType::U64 => 8,
            DataType::F64 => 8,
            DataType::Nothing => 0,
            DataType::Bool => 1,
            DataType::Utf8 => {
                self.pc += 1;
                let (data_type, value) = self.get_value_length();
                if data_type != DataType::U32 {
                    panic!("bad utf8 value length")
                }

                let (string_length, _) = bytes_to_data(&DataType::U32, &value);
                if let Value::U32(val) = string_length {
                    val.value as usize
                } else {
                    panic!("Unexpected value type for string length");
                }
            } // hardcoded for the moment
            _ => {
                panic!("Unsupported datatype")
            }
        };

        if (self.pc + value_length) >= self.bytecode.len() {
            panic!("Invalid value size at position {}", self.pc + 1);
        };

        let value_bytes = self.bytecode[self.pc + 1..self.pc + 1 + value_length].to_vec();
        self.pc += value_length;

        (data_type, value_bytes)
    }

    fn read_offset(bytes: &[u8]) -> i32 {
        let arr: [u8; 4] = bytes.try_into().expect("slice with incorrect length");
        i32::from_le_bytes(arr)
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
        println!("\n--- BYTECODE ----------\n");
        let mut pc = 0;
        let mut target_pc = 0;

        let string_offset = self.bytecode.len().to_string();
        while pc < self.bytecode.len() {
            let index = (pc + 1).to_string();
            let mut counter = 0;
            let printable_index = string_offset
                .chars()
                .map(|_| {
                    let mut result = "".to_string();
                    if let Some(char) = index.chars().nth(counter) {
                        result = char.to_string();
                    } else {
                        result = " ".to_string();
                    }
                    counter += 1;
                    return result;
                })
                .collect::<String>();

            if pc >= target_pc {
                // print instruction
                let (instruction, offset) = Translator::get_instruction(pc, &self.bytecode);
                let raw_instruction = format!("{}|    {:#?}", printable_index, self.bytecode[pc]);
                println!("{}-----{}", raw_instruction, instruction.get_type());
                println!(
                    "{}     {}",
                    raw_instruction
                        .chars()
                        .map(|_| " ".to_string())
                        .collect::<String>(),
                    Translator::get_instruction_info(&instruction)
                );
                // + 1  the normal iteration increment over the bytecode
                target_pc = pc + offset + 1;
            } else {
                // print bytecode index
                println!("{}|    {:#?}", printable_index, self.bytecode[pc]);
            }

            pc += 1;
        }
        //println!("\n--- BYTECODE INSTRUCTIONS ----------\n");
        //println!("{:#?}", Translator::new(self.bytecode.clone()).translate());
    }
}
