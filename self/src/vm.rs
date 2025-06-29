use crate::core::error::InvalidBinaryOperation;
use crate::core::error::VMErrorType;
use crate::core::execution::VMExecutionResult;
use crate::core::handlers::ai_handler::ai_handler;
use crate::core::handlers::call_handler::call_handler;
use crate::core::handlers::foreign_handlers::ForeignHandlers;
use crate::core::handlers::print_handler::print_handler;
use crate::heap::Heap;
use crate::heap::HeapObject;
use crate::heap::HeapRef;
use crate::opcodes::DataType;
use crate::opcodes::Opcode;
use crate::translator::Translator;
use crate::types::object::func::Function;
use crate::types::raw::RawValue;
use crate::types::raw::{bool::Bool, f64::F64, i32::I32, i64::I64, u32::U32, u64::U64, utf8::Utf8};
use crate::utils::foreign_handlers_utils::get_foreign_handlers;

use super::stack::*;
use super::types::*;

pub struct Vm {
    operand_stack: Vec<OperandsStackValue>,
    call_stack: CallStack,
    heap: Heap,
    bytecode: Vec<u8>,
    pc: usize,
    handlers: ForeignHandlers,
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
            call_stack: CallStack::new(),
            heap: Heap::new(),
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
                    let (value, printable_value) = self.bytes_to_data(&data_type, &value_bytes);
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
                    // identifier
                    let (identifier_data_type, identifier_bytes) = self.get_value_length();
                    if identifier_data_type != DataType::Utf8 {
                        panic!("Identifier type should be a string encoded as utf8")
                    }
                    let identifier_name = String::from_utf8(identifier_bytes)
                        .expect("Identifier bytes should be valid UTF-8");

                    let identifier_value = self.call_stack.resolve(&identifier_name);
                    if let Some(v) = identifier_value {
                        self.push_to_stack(v, Some(identifier_name.clone()));
                        if debug {
                            println!("LOAD_VAR <- {identifier_name}");
                        }
                    } else {
                        // should be handled with ego errors
                        panic!("{identifier_name} is not defined")
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
                        self.call_stack
                            .put_to_frame(identifier_name.clone(), v.value);
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
                Opcode::JumpIfFalse => {
                    let offset = Vm::read_offset(&self.bytecode[self.pc + 1..self.pc + 5]);
                    self.pc += 4;

                    let condition = self.operand_stack.pop();
                    if condition.is_none() {
                        panic!("stack underflow");
                    };

                    let condition = condition.unwrap();
                    match condition.value {
                        Value::RawValue(v) => match v {
                            RawValue::Bool(execute_if) => {
                                if debug {
                                    println!("JUMP_IF_FALSE <- {:?}({})", execute_if.value, offset);
                                }
                                if !execute_if.value {
                                    self.pc += offset as usize;
                                }
                            }
                            _ => panic!("invalid expression type as condition to jump"),
                        },
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
                    self.pc += 1; // consume print opcode
                    let args = self.get_function_call_args();
                    let mut resolved_args = Vec::new();
                    for val in args {
                        match self.value_to_string(val) {
                            Ok(v) => resolved_args.push(v),
                            Err(e) => return VMExecutionResult::terminate_with_errors(e),
                        }
                    }
                    print_handler(resolved_args, debug, false);
                }
                Opcode::Println => {
                    self.pc += 1; // consume print opcode
                    let args = self.get_function_call_args();
                    let mut resolved_args = Vec::new();
                    for val in args {
                        match self.value_to_string(val) {
                            Ok(v) => resolved_args.push(v),
                            Err(e) => return VMExecutionResult::terminate_with_errors(e),
                        }
                    }
                    print_handler(resolved_args, debug, true);
                }
                Opcode::FuncDec => {
                    // skip FuncDec opcode
                    if self.pc + 1 >= self.bytecode.len() {
                        panic!(
                            "Invalid FUNCTION_DECLARATION instruction at position {}.",
                            self.pc
                        );
                    } else {
                        self.pc += 1;
                    }

                    // identifier
                    let (identifier_data_type, identifier_bytes) = self.get_value_length();
                    if identifier_data_type != DataType::Utf8 {
                        panic!("Identifier type should be a string encoded as utf8")
                    }
                    let identifier_name = String::from_utf8(identifier_bytes)
                        .expect("Identifier bytes should be valid UTF-8");

                    // handle body
                    // function body length
                    if self.pc + 4 >= self.bytecode.len() {
                        panic!("Invalid FUNC_DEC instruction at position {}", self.pc);
                    }

                    let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
                    let body_length = u32::from_le_bytes(
                        value_bytes.try_into().expect("Provided value is incorrect"),
                    ) as usize;
                    self.pc += 4;
                    self.pc += 1; // to get next opcode

                    let body_boytecode = self.bytecode[self.pc..self.pc + body_length].to_vec();
                    self.pc += body_length;

                    // allocate function on the heap
                    let func_obj = HeapObject::Function(Function::new(
                        identifier_name.clone(),
                        body_boytecode,
                    ));
                    let func_ref = self.heap.allocate(func_obj);

                    // make accesible on the current context
                    self.call_stack
                        .put_to_frame(identifier_name, Value::HeapRef(func_ref));
                }
                // PROBABLY BEHIND THE SCENE TO AVOID RUNTIME ERRORS
                // WE SHOULD WRAP AI OPCODE WITHIN A BOOLEAN CAST OPCODE
                Opcode::Ai => {
                    self.pc += 1; // consume print opcode
                    let args = self.get_function_call_args();
                    let mut resolved_args = Vec::new();
                    for val in args {
                        match self.value_to_string(val) {
                            Ok(v) => resolved_args.push(v),
                            Err(e) => return VMExecutionResult::terminate_with_errors(e),
                        }
                    }
                    let value = ai_handler(resolved_args, debug);
                    if let Some(v) = value {
                        let answer_type = v.0;
                        let value = v.1;

                        // here the <value>'s should be well
                        // formatted since we verify their format
                        // on the ai_handler
                        match answer_type.as_str() {
                            "bool" => {
                                let bool_value: bool = value.parse().unwrap();
                                self.push_to_stack(
                                    Value::RawValue(RawValue::Bool(Bool::new(bool_value))),
                                    Some("AI_INFERRED".to_string()),
                                );
                            }
                            "string" => {
                                // heap allocated
                                let heap_ref = self.heap.allocate(HeapObject::String(value));
                                self.push_to_stack(
                                    Value::HeapRef(heap_ref),
                                    Some("AI_INFERRED".to_string()),
                                );
                            }
                            "number" => {
                                // number inferred
                                let num_value: f64 = value.parse().unwrap();
                                self.push_to_stack(
                                    Value::RawValue(RawValue::F64(F64::new(num_value))),
                                    Some("AI_INFERRED".to_string()),
                                );
                            }
                            "nothing" => {
                                self.push_to_stack(
                                    Value::RawValue(RawValue::Nothing),
                                    Some("AI_INFERRED".to_string()),
                                );
                            }
                            _ => {
                                self.push_to_stack(
                                    Value::RawValue(RawValue::Nothing),
                                    Some("AI_INFERRED".to_string()),
                                );
                            }
                        };
                    } else {
                        self.push_to_stack(
                            Value::RawValue(RawValue::Nothing),
                            Some("AI_INFERRED".to_string()),
                        );
                    }
                }
                Opcode::Add => {
                    // execution
                    let right_operand = self.operand_stack.pop();
                    let left_operand = self.operand_stack.pop();

                    if left_operand.is_none() || right_operand.is_none() {
                        panic!("Operands stack underflow");
                    };

                    let operands_stack_values = (left_operand.unwrap(), right_operand.unwrap());

                    let error = self.run_binary_expression("+", operands_stack_values);
                    if let Some(err) = error {
                        return VMExecutionResult::terminate_with_errors(err);
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

                    let operands_stack_values = (left_operand.unwrap(), right_operand.unwrap());

                    let error = self.run_binary_expression("-", operands_stack_values);
                    if let Some(err) = error {
                        return VMExecutionResult::terminate_with_errors(err);
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

                    let operands_stack_values = (left_operand.unwrap(), right_operand.unwrap());

                    let error = self.run_binary_expression("*", operands_stack_values);
                    if let Some(err) = error {
                        return VMExecutionResult::terminate_with_errors(err);
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

                    let operands_stack_values = (left_operand.unwrap(), right_operand.unwrap());

                    let error = self.run_binary_expression("/", operands_stack_values);
                    if let Some(err) = error {
                        return VMExecutionResult::terminate_with_errors(err);
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

                    let operands_stack_values = (left_operand.unwrap(), right_operand.unwrap());

                    let error = self.run_binary_expression(">", operands_stack_values);
                    if let Some(err) = error {
                        return VMExecutionResult::terminate_with_errors(err);
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

                    let operands_stack_values = (left_operand.unwrap(), right_operand.unwrap());

                    let error = self.run_binary_expression("<", operands_stack_values);
                    if let Some(err) = error {
                        return VMExecutionResult::terminate_with_errors(err);
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

                    let operands_stack_values = (left_operand.unwrap(), right_operand.unwrap());

                    let error = self.run_binary_expression("==", operands_stack_values);
                    if let Some(err) = error {
                        return VMExecutionResult::terminate_with_errors(err);
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

                    let operands_stack_values = (left_operand.unwrap(), right_operand.unwrap());

                    let error = self.run_binary_expression("!=", operands_stack_values);
                    if let Some(err) = error {
                        return VMExecutionResult::terminate_with_errors(err);
                    }

                    self.pc += 1;
                }
                Opcode::Call => {
                    self.pc += 1; // consume call opcode
                    let args = self.get_function_call_args();
                    let mut resolved_args = Vec::new();
                    for val in args {
                        match self.value_to_string(val) {
                            Ok(v) => resolved_args.push(v),
                            Err(e) => return VMExecutionResult::terminate_with_errors(e),
                        }
                    }
                    if debug {
                        println!("CALL -> {}", resolved_args[0].to_string())
                    }
                    call_handler(&self.handlers, resolved_args);
                }
                _ => {
                    println!("unhandled opcode");
                    self.pc += 1;
                }
            };
        }

        VMExecutionResult::terminate()
    }

    fn run_binary_expression(
        &mut self,
        operator: &str,
        operands: (OperandsStackValue, OperandsStackValue),
    ) -> Option<VMErrorType> {
        let left = operands.0;
        let right = operands.1;

        let value: Value;
        // cloned here, to be able to use later on
        // different VMErrors
        match (left.value, right.value.clone()) {
            (Value::RawValue(l), Value::RawValue(r)) => {
                let result_value = match (l, r) {
                    (RawValue::I32(l), RawValue::I32(r)) => match operator {
                        "+" => RawValue::I32(I32::new(l.value + r.value)),
                        "-" => RawValue::I32(I32::new(l.value - r.value)),
                        "*" => RawValue::I32(I32::new(l.value * r.value)),
                        "/" => RawValue::I32(I32::new(l.value / r.value)),
                        ">" => RawValue::Bool(Bool::new(l.value > r.value)),
                        "<" => RawValue::Bool(Bool::new(l.value < r.value)),
                        "==" => RawValue::Bool(Bool::new(l.value == r.value)),
                        "!=" => RawValue::Bool(Bool::new(l.value != r.value)),
                        _ => {
                            panic!("operator not implemented")
                        }
                    },
                    (RawValue::I64(l), RawValue::I64(r)) => match operator {
                        "+" => RawValue::I64(I64::new(l.value + r.value)),
                        "-" => RawValue::I64(I64::new(l.value - r.value)),
                        "*" => RawValue::I64(I64::new(l.value * r.value)),
                        "/" => RawValue::I64(I64::new(l.value / r.value)),
                        ">" => RawValue::Bool(Bool::new(l.value > r.value)),
                        "<" => RawValue::Bool(Bool::new(l.value < r.value)),
                        "==" => RawValue::Bool(Bool::new(l.value == r.value)),
                        "!=" => RawValue::Bool(Bool::new(l.value != r.value)),
                        _ => {
                            panic!("operator not implemented in i64")
                        }
                    },
                    (RawValue::U32(l), RawValue::U32(r)) => match operator {
                        "+" => RawValue::U32(U32::new(l.value + r.value)),
                        "-" => RawValue::U32(U32::new(l.value - r.value)),
                        "*" => RawValue::U32(U32::new(l.value * r.value)),
                        "/" => RawValue::U32(U32::new(l.value / r.value)),
                        ">" => RawValue::Bool(Bool::new(l.value > r.value)),
                        "<" => RawValue::Bool(Bool::new(l.value < r.value)),
                        "==" => RawValue::Bool(Bool::new(l.value == r.value)),
                        "!=" => RawValue::Bool(Bool::new(l.value != r.value)),
                        _ => {
                            panic!("operator not implemented in u32")
                        }
                    },
                    (RawValue::U64(l), RawValue::U64(r)) => match operator {
                        "+" => RawValue::U64(U64::new(l.value + r.value)),
                        "-" => RawValue::U64(U64::new(l.value - r.value)),
                        "*" => RawValue::U64(U64::new(l.value * r.value)),
                        "/" => RawValue::U64(U64::new(l.value / r.value)),
                        ">" => RawValue::Bool(Bool::new(l.value > r.value)),
                        "<" => RawValue::Bool(Bool::new(l.value < r.value)),
                        "==" => RawValue::Bool(Bool::new(l.value == r.value)),
                        "!=" => RawValue::Bool(Bool::new(l.value != r.value)),
                        _ => {
                            panic!("operator not implemented in u64")
                        }
                    },
                    (RawValue::F64(l), RawValue::F64(r)) => match operator {
                        "+" => RawValue::F64(F64::new(l.value + r.value)),
                        "-" => RawValue::F64(F64::new(l.value - r.value)),
                        "*" => RawValue::F64(F64::new(l.value * r.value)),
                        "/" => RawValue::F64(F64::new(l.value / r.value)),
                        ">" => RawValue::Bool(Bool::new(l.value > r.value)),
                        "<" => RawValue::Bool(Bool::new(l.value < r.value)),
                        "==" => RawValue::Bool(Bool::new(l.value == r.value)),
                        "!=" => RawValue::Bool(Bool::new(l.value != r.value)),
                        _ => {
                            panic!("operator not implemented in f64")
                        }
                    },
                    (RawValue::Nothing, RawValue::Nothing) => {
                        return Some(VMErrorType::InvalidBinaryOperation(
                            InvalidBinaryOperation {
                                left: DataType::Nothing,
                                right: DataType::Nothing,
                                operator: operator.to_string(),
                            },
                        ))
                    }
                    (RawValue::Utf8(_), RawValue::Utf8(_)) => {
                        return Some(VMErrorType::InvalidBinaryOperation(
                            InvalidBinaryOperation {
                                left: DataType::Utf8,
                                right: DataType::Utf8,
                                operator: operator.to_string(),
                            },
                        ))
                    }
                    (RawValue::Bool(_), RawValue::Bool(_)) => {
                        return Some(VMErrorType::InvalidBinaryOperation(
                            InvalidBinaryOperation {
                                left: DataType::Bool,
                                right: DataType::Bool,
                                operator: operator.to_string(),
                            },
                        ))
                    }
                    _ => return Some(VMErrorType::TypeCoercionError(right)),
                };

                value = Value::RawValue(result_value);
            }
            (Value::HeapRef(l), Value::HeapRef(r)) => {
                // here implement binary operations between different
                // types once the HeapRef is resolved to the actual value
                let l_heap_object = self.resolve_heap_ref(l);
                let r_heap_object = self.resolve_heap_ref(r);

                let result_value = match (l_heap_object, r_heap_object) {
                    (HeapObject::String(left_string), HeapObject::String(right_string)) => {
                        match operator {
                            "+" => {
                                let result_string = format!("{left_string}{right_string}");
                                self.heap.allocate(HeapObject::String(result_string))
                            }
                            _ => {
                                return Some(VMErrorType::InvalidBinaryOperation(
                                    InvalidBinaryOperation {
                                        left: DataType::Utf8,
                                        right: DataType::Utf8,
                                        operator: operator.to_string(),
                                    },
                                ))
                            }
                        }
                    } // when more heap type exists implement here a
                    _ => {
                        return Some(VMErrorType::InvalidBinaryOperation(
                            // we should (probably) implement a system to refer to functions
                            // data type either creating a new type RuntimeType or extending
                            // DataType
                            InvalidBinaryOperation {
                                left: DataType::Unknown,
                                right: DataType::Unknown,
                                operator: operator.to_string(),
                            },
                        ));
                    }
                };

                value = Value::HeapRef(result_value);
            }
            (Value::HeapRef(_), Value::RawValue(_)) => {
                return Some(VMErrorType::TypeCoercionError(right))
            }
            (Value::RawValue(_), Value::HeapRef(_)) => {
                return Some(VMErrorType::TypeCoercionError(right))
            }
        }

        self.push_to_stack(value, None);
        None
    }

    fn resolve_heap_ref(&self, address: HeapRef) -> &HeapObject {
        if let Some(addr) = self.heap.get(address) {
            return addr;
        } else {
            panic!("ref is not defined in the heap")
        }
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

                let (string_length, _) = self.bytes_to_data(&DataType::U32, &value);
                if let Value::RawValue(RawValue::U32(val)) = string_length {
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

    pub fn bytes_to_data(&mut self, data_type: &DataType, value: &Vec<u8>) -> (Value, String) {
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
                Value::RawValue(RawValue::I32(I32::new(value)))
            }
            DataType::I64 => {
                let value = i64::from_le_bytes(
                    value
                        .as_slice()
                        .try_into()
                        .expect("Provided value is incorrect"),
                );
                printable_value = value.to_string();
                Value::RawValue(RawValue::I64(I64::new(value)))
            }
            DataType::U32 => {
                let value = u32::from_le_bytes(
                    value
                        .as_slice()
                        .try_into()
                        .expect("Provided value is incorrect"),
                );
                printable_value = value.to_string();
                Value::RawValue(RawValue::U32(U32::new(value)))
            }
            DataType::U64 => {
                let value = u64::from_le_bytes(
                    value
                        .as_slice()
                        .try_into()
                        .expect("Provided value is incorrect"),
                );
                printable_value = value.to_string();
                Value::RawValue(RawValue::U64(U64::new(value)))
            }
            DataType::F64 => {
                let value = f64::from_le_bytes(
                    value
                        .as_slice()
                        .try_into()
                        .expect("Provided value is incorrect"),
                );
                printable_value = value.to_string();
                Value::RawValue(RawValue::F64(F64::new(value)))
            }
            DataType::Utf8 => {
                let value =
                    String::from_utf8(value.clone()).expect("Provided value is not valid UTF-8");
                printable_value = value.to_string();

                let value_ref = self.heap.allocate(HeapObject::String(value));
                Value::HeapRef(value_ref)
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
                Value::RawValue(RawValue::Bool(Bool::new(value)))
            }
            DataType::Nothing => {
                printable_value = "nothing".to_string();
                Value::RawValue(RawValue::Nothing)
            }
            _ => {
                panic!("Unsupported type to get data from")
            }
        };

        (value, printable_value)
    }

    fn value_to_string(&mut self, value: Value) -> Result<String, VMErrorType> {
        match value {
            Value::RawValue(x) => Ok(x.to_string()),
            Value::HeapRef(x) => match self.heap.get(x) {
                Some(x) => Ok(x.to_string()),
                None => {
                    // identifier not defined
                    //Err(VMErrorType::Iden)
                    panic!("idenifier not defined")
                }
            },
        }
    }

    fn read_offset(bytes: &[u8]) -> i32 {
        let arr: [u8; 4] = bytes.try_into().expect("slice with incorrect length");
        i32::from_le_bytes(arr)
    }

    fn get_function_call_args(&mut self) -> Vec<Value> {
        // get u32 value. 4 bytes based on the type plus the current
        let value_length = 3;
        if self.pc + value_length >= self.bytecode.len() {
            panic!("Invalid instruction at position {}", self.pc);
        }

        let value_bytes = &self.bytecode[self.pc..self.pc + 4];
        let number_of_args =
            u32::from_le_bytes(value_bytes.try_into().expect("Provided value is incorrect"));
        self.pc += 4; // 4 => 3 + 1 extra to leave the pc in the next opcode

        // execution
        let args = self.get_stack_values(&number_of_args);
        args
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
        self.operand_stack
            .push(OperandsStackValue { value, origin });
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
