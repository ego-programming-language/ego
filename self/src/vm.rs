use crate::core::error::struct_errors::StructError;
use crate::core::error::InvalidBinaryOperation;
use crate::core::error::VMErrorType;
use crate::core::execution::VMExecutionResult;
use crate::core::handlers::call_handler::call_handler;
use crate::core::handlers::foreign_handlers::ForeignHandlers;
use crate::core::handlers::print_handler::print_handler;
use crate::heap::Heap;
use crate::memory::Handle;
use crate::memory::MemObject;
use crate::memory::MemoryManager;
use crate::opcodes::DataType;
use crate::opcodes::Opcode;
use crate::std::bootstrap_default_lib;
use crate::std::vector;
use crate::std::{generate_native_module, get_native_module_type};
use crate::types::object::func::Engine;
use crate::types::object::func::Function;
use crate::types::object::structs::StructDeclaration;
use crate::types::object::structs::StructLiteral;
use crate::types::object::vector::Vector;
use crate::types::object::BoundAccess;
use crate::types::raw::RawValue;
use crate::types::raw::{bool::Bool, f64::F64, i32::I32, i64::I64, u32::U32, u64::U64};
use crate::utils::foreign_handlers_utils::get_foreign_handlers;
use std::collections::HashMap;
use std::path::Path;

use super::stack::*;
use super::types::*;

pub struct Vm {
    operand_stack: Vec<OperandsStackValue>,
    pub call_stack: CallStack,
    pub heap: Heap,
    pub memory: MemoryManager,
    bytecode: Vec<u8>,
    pc: usize,
    handlers: HashMap<String, Handle>,
    ffi_handlers: ForeignHandlers,
}

impl Vm {
    pub fn new(bytecode: Vec<u8>) -> Vm {
        //let mut translator = Translator::new(bytecode);
        //let instructions = translator.translate();

        // load ffi_handlers
        let mut ffi_handlers = ForeignHandlers::new();
        let foreign_handlers = get_foreign_handlers();

        if let Some(loaded_handlers) = foreign_handlers {
            for handler in loaded_handlers.functions {
                ffi_handlers.add(handler);
            }
        }

        Vm {
            operand_stack: vec![],
            call_stack: CallStack::new(),
            heap: Heap::new(),
            memory: MemoryManager::new(),
            bytecode,
            pc: 0,
            handlers: HashMap::new(),
            ffi_handlers,
        }
    }

    pub fn run(&mut self, args: &Vec<String>) -> VMExecutionResult {
        let debug = args.contains(&"-d".to_string());
        if debug {
            println!("last PC value: {}", self.bytecode.len());
            println!("-");
        }

        // load builtin handlers
        let raw_handlers = bootstrap_default_lib();
        let mut handlers = HashMap::new();
        for (handler_name, handler_obj) in raw_handlers {
            let obj_handle = self.memory.alloc(handler_obj);
            handlers.insert(handler_name, obj_handle);
        }
        self.handlers = handlers;

        self.run_bytecode(debug)
    }

    fn run_bytecode(&mut self, debug: bool) -> VMExecutionResult {
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
                        return VMExecutionResult::terminate_with_errors(
                            VMErrorType::UndeclaredIdentifierError(identifier_name),
                            self,
                        );
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
                        let printable_value = v.value.to_string(self);
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
                            Err(e) => return VMExecutionResult::terminate_with_errors(e, self),
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
                            Err(e) => return VMExecutionResult::terminate_with_errors(e, self),
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

                    // parameters
                    if self.pc + 4 >= self.bytecode.len() {
                        panic!("Invalid FUNC_DEC instruction at position {}", self.pc);
                    }

                    let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
                    let parameters_length = u32::from_le_bytes(
                        value_bytes.try_into().expect("Provided value is incorrect"),
                    ) as usize;
                    // get params names from the stack
                    let params_values = self.get_stack_values(&(parameters_length as u32));
                    let params_names: Vec<String> = params_values
                        .iter()
                        .map(|val| {
                            match val {
                                Value::Handle(r) => match self.memory.resolve(&r) {
                                    MemObject::String(s) => s.clone(),
                                    _ => {
                                        // TODO: use self-vm errors sytem
                                        panic!("Invalid param type for a function declaration")
                                    }
                                },
                                _ => {
                                    // TODO: use self-vm errors sytem
                                    panic!("Invalid param type for a function declaration")
                                }
                            }
                        })
                        .collect();

                    self.pc += 4;

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

                    let body_bytecode = self.bytecode[self.pc..self.pc + body_length].to_vec();
                    self.pc += body_length;

                    // allocate function on the heap
                    let func_obj = MemObject::Function(Function::new(
                        identifier_name.clone(),
                        params_names,
                        Engine::Bytecode(body_bytecode),
                    ));
                    let func_handle = self.memory.alloc(func_obj);

                    // make accesible on the current context
                    self.call_stack
                        .put_to_frame(identifier_name, Value::Handle(func_handle));
                }
                Opcode::StructDec => {
                    // skip StructDec opcode
                    self.pc += 1;

                    // identifier
                    let (identifier_data_type, identifier_bytes) = self.get_value_length();
                    if identifier_data_type != DataType::Utf8 {
                        // TODO: use self-vm errors
                        panic!("Identifier type should be a string encoded as utf8")
                    }

                    // TODO: use self-vm errors
                    let identifier_name = String::from_utf8(identifier_bytes)
                        .expect("Identifier bytes should be valid UTF-8");

                    // read fields number
                    self.pc += 1;
                    let fields_num = Vm::read_offset(&self.bytecode[self.pc..self.pc + 4]);
                    self.pc += 4;

                    // struct fields [raw_string][type][raw_string][type]
                    //               (x)B        1B    (x)B        1B
                    let mut counter = 0;
                    let mut fields = vec![];
                    while counter < fields_num {
                        // field
                        let (field_data_type, field_bytes) = self.get_value_length();
                        if field_data_type != DataType::Utf8 {
                            // TODO: use self-vm errors
                            panic!("Identifier type should be a string encoded as utf8")
                        }
                        let field_name = String::from_utf8(field_bytes)
                            .expect("Field bytes should be valid UTF-8"); // TODO: use self-vm errors
                        self.pc += 1;

                        // annotation
                        let annotation = DataType::to_opcode(self.bytecode[self.pc]);
                        self.pc += 1;

                        fields.push((field_name, annotation));
                        counter += 1;
                    }

                    // struct declaration
                    let struct_declaration =
                        StructDeclaration::new(identifier_name.clone(), fields);
                    // push to declaration heap
                    let heap_handle = self
                        .memory
                        .alloc(MemObject::StructDeclaration(struct_declaration));
                    self.call_stack
                        .put_to_frame(identifier_name, Value::Handle(heap_handle));
                }
                Opcode::GetProperty => {
                    let values = self.get_stack_values(&2);
                    let (object_handle, property_handle) = match (&values[0], &values[1]) {
                        // (Value::HeapRef(obj_ref), Value::HeapRef(prop_ref)) => (
                        //     self.resolve_heap_ref(obj_ref.clone()),
                        //     self.resolve_heap_ref(prop_ref.clone()),
                        // ),
                        (Value::Handle(obj_handle), Value::Handle(prop_handle)) => {
                            (obj_handle, prop_handle)
                        }
                        // TODO: use self-vm errors
                        // here we should handle if a function returns an
                        // nothing istead of a struct
                        _ => panic!("Expected two Handle values for <get_property> opcode"),
                    };

                    let object = self.memory.resolve(object_handle);
                    let property = self.memory.resolve(property_handle);

                    if debug {
                        println!(
                            "GET_PROPERTY <- {}({:?})",
                            object.to_string(self),
                            property.to_string(self)
                        );
                    }

                    if let MemObject::String(property_key) = property {
                        match object {
                            MemObject::StructLiteral(x) => {
                                let value = x.property_access(&property_key);
                                if let Some(prop) = value {
                                    let bound_access =
                                        BoundAccess::new(object_handle.clone(), Box::new(prop));
                                    self.push_to_stack(
                                        Value::BoundAccess(bound_access),
                                        Some(object.to_string(self)),
                                    );
                                } else {
                                    return VMExecutionResult::terminate_with_errors(
                                        VMErrorType::Struct(StructError::FieldNotFound {
                                            field: property_key.to_string(),
                                            struct_type: object.to_string(self),
                                        }),
                                        self,
                                    );
                                }
                            }
                            MemObject::NativeStruct(x) => {
                                let value = x.property_access(&property_key);
                                if let Some(prop) = value {
                                    let bound_access =
                                        BoundAccess::new(object_handle.clone(), Box::new(prop));
                                    self.push_to_stack(
                                        Value::BoundAccess(bound_access),
                                        Some(object.to_string(self)),
                                    );
                                } else {
                                    return VMExecutionResult::terminate_with_errors(
                                        VMErrorType::Struct(StructError::FieldNotFound {
                                            field: property_key.to_string(),
                                            struct_type: object.to_string(self),
                                        }),
                                        self,
                                    );
                                }
                            }
                            MemObject::Vector(x) => {
                                let value = x.property_access(&property_key);
                                if let Some(prop) = value {
                                    let bound_access =
                                        BoundAccess::new(object_handle.clone(), Box::new(prop));
                                    self.push_to_stack(
                                        Value::BoundAccess(bound_access),
                                        Some(object.to_string(self)),
                                    );
                                } else {
                                    return VMExecutionResult::terminate_with_errors(
                                        VMErrorType::Struct(StructError::FieldNotFound {
                                            field: property_key.to_string(),
                                            struct_type: object.to_string(self),
                                        }),
                                        self,
                                    );
                                }
                            }
                            _ => {
                                panic!("<get_property> opcode must be used on a Struct like type")
                            }
                        }
                    } else {
                        // TODO: use self-vm errors
                        panic!("Struct literal field must be indexed by string")
                    }

                    self.pc += 1;
                }
                Opcode::Call => {
                    self.pc += 1;
                    let args = self.get_function_call_args();
                    let callee_value = self.get_stack_values(&1);
                    let ((caller_obj, caller_handle), callee_handle): (
                        (&MemObject, Handle),
                        Option<Handle>,
                    ) = match callee_value[0].clone() {
                        // Value::HeapRef(_ref) => {
                        //     let owned_ref = _ref.clone();
                        //     ((self.resolve_heap_ref(_ref), owned_ref), None)
                        // }
                        Value::Handle(handle) => ((self.memory.resolve(&handle), handle), None),
                        Value::BoundAccess(b) => {
                            if let Value::Handle(callee_handle) = b.property.as_ref() {
                                (
                                    (self.memory.resolve(&b.object), b.object),
                                    Some(callee_handle.clone()),
                                )
                            } else {
                                // nested bound accesses
                                panic!("Invalid type for callee string")
                            }
                        }
                        _ => {
                            // TODO: use self-vm error system
                            panic!("Invalid type for callee string")
                        }
                    };

                    match caller_obj {
                        // FOR NAMED FUNCTIONS ACCESS
                        MemObject::String(identifier_name) => {
                            if debug {
                                println!("CALL -> {}", identifier_name.to_string())
                            };
                            match identifier_name.as_str() {
                                // BUILTIN FUNCTIONS
                                "eprintln" => {
                                    println!("------ eprintln")
                                }
                                // RUNTIME DEFINED FUNCTIONS
                                _ => {
                                    // get the identifier from the heap
                                    let value = if let Some(value) =
                                        self.call_stack.resolve(&identifier_name)
                                    {
                                        value
                                    } else {
                                        return VMExecutionResult::terminate_with_errors(
                                            VMErrorType::UndeclaredIdentifierError(
                                                identifier_name.clone(),
                                            ),
                                            self,
                                        );
                                    };

                                    match value {
                                        Value::Handle(v) => {
                                            // clone heap_object to be able to mutate the
                                            // vm state
                                            let heap_object = self.memory.resolve(&v);
                                            if let MemObject::Function(func) = heap_object {
                                                let func = func.clone();
                                                let exec_result = self.run_function(
                                                    &func,
                                                    Some(caller_handle),
                                                    args.clone(),
                                                    debug,
                                                );
                                                if exec_result.error.is_some() {
                                                    return VMExecutionResult::terminate_with_errors(
                                                                exec_result.error.unwrap().error_type,
                                                                self
                                                            );
                                                }
                                                if let Some(returned_value) = &exec_result.result {
                                                    self.push_to_stack(
                                                        returned_value.clone(),
                                                        Some(func.identifier.clone()),
                                                    );
                                                }
                                            } else {
                                                return VMExecutionResult::terminate_with_errors(
                                                    VMErrorType::NotCallableError(
                                                        identifier_name.clone(),
                                                    ),
                                                    self,
                                                );
                                            }
                                        }
                                        _ => {
                                            return VMExecutionResult::terminate_with_errors(
                                                VMErrorType::NotCallableError(
                                                    identifier_name.clone(),
                                                ),
                                                self,
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        // FOR STRUCTS CALLABLE MEMBERS
                        MemObject::StructLiteral(caller) => {
                            let callee_handle = if let Some(c) = callee_handle {
                                c
                            } else {
                                // TODO: use self-vm error system
                                panic!("callee is not defined for a struct as function caller")
                            };

                            let callee = self.memory.resolve(&callee_handle);
                            if let MemObject::Function(func) = callee {
                                let func = func.clone();
                                let exec_result = self.run_function(
                                    &func,
                                    Some(caller_handle),
                                    args.clone(),
                                    debug,
                                );
                                if exec_result.error.is_some() {
                                    return VMExecutionResult::terminate_with_errors(
                                        exec_result.error.unwrap().error_type,
                                        self,
                                    );
                                }
                                if let Some(returned_value) = &exec_result.result {
                                    self.push_to_stack(
                                        returned_value.clone(),
                                        Some(func.identifier.clone()),
                                    );
                                }
                            } else {
                                return VMExecutionResult::terminate_with_errors(
                                    VMErrorType::NotCallableError(caller.struct_type.clone()),
                                    self,
                                );
                            }
                        }

                        // FOR NATIVE_STRUCTS CALLABLE MEMBERS
                        MemObject::NativeStruct(caller) => {
                            let callee_handle = if let Some(c) = callee_handle {
                                c
                            } else {
                                // TODO: use self-vm error system
                                panic!("callee is not defined for a struct as function caller")
                            };

                            let callee = self.memory.resolve(&callee_handle);
                            if let MemObject::Function(func) = callee {
                                let func = func.clone();
                                let exec_result =
                                    // instead of none callee_handle
                                    self.run_function(&func, Some(caller_handle), args.clone(), debug);
                                if exec_result.error.is_some() {
                                    return VMExecutionResult::terminate_with_errors(
                                        exec_result.error.unwrap().error_type,
                                        self,
                                    );
                                }
                                if let Some(returned_value) = &exec_result.result {
                                    self.push_to_stack(
                                        returned_value.clone(),
                                        Some(func.identifier.clone()),
                                    );
                                }
                            } else {
                                return VMExecutionResult::terminate_with_errors(
                                    VMErrorType::NotCallableError(caller.to_string()),
                                    self,
                                );
                            }
                        }

                        // FOR VECTOR CALLABLE MEMBERS
                        MemObject::Vector(caller) => {
                            let callee_handle = if let Some(c) = callee_handle {
                                c
                            } else {
                                // TODO: use self-vm error system
                                panic!("callee is not defined for a vec as a function caller")
                            };

                            let callee = self.memory.resolve(&callee_handle);
                            if let MemObject::Function(func) = callee {
                                let func = func.clone();
                                let exec_result = self.run_function(
                                    &func,
                                    Some(caller_handle),
                                    args.clone(),
                                    debug,
                                );
                                if exec_result.error.is_some() {
                                    return VMExecutionResult::terminate_with_errors(
                                        exec_result.error.unwrap().error_type,
                                        self,
                                    );
                                }
                                if let Some(returned_value) = &exec_result.result {
                                    self.push_to_stack(
                                        returned_value.clone(),
                                        Some(func.identifier.clone()),
                                    );
                                }
                            } else {
                                return VMExecutionResult::terminate_with_errors(
                                    VMErrorType::NotCallableError(caller.to_string(self)),
                                    self,
                                );
                            }
                        }
                        _ => {
                            panic!("Invalid type for callee string")
                        }
                    }
                }
                Opcode::Import => {
                    let values = self.get_stack_values(&1);
                    let module_name_value = values[0].clone();
                    let mod_bytecode_length =
                        Vm::read_offset(&self.bytecode[self.pc + 1..self.pc + 5]);
                    self.pc += 4;

                    if let Value::Handle(mod_handle) = module_name_value {
                        let module_name = self.memory.resolve(&mod_handle).to_string(self);
                        let native_module = get_native_module_type(module_name.as_str());
                        // native module
                        if let Some(nm) = native_module {
                            // load native module fields
                            let module_def = generate_native_module(nm);
                            let mut module_fields = HashMap::new();
                            for field in module_def.1 {
                                let field_handle = self.memory.alloc(field.1);
                                module_fields.insert(field.0, Value::Handle(field_handle));
                            }

                            // create the native module struct
                            let module_struct = StructLiteral::new(module_def.0, module_fields);
                            let module_struct_handle =
                                self.memory.alloc(MemObject::StructLiteral(module_struct));

                            self.call_stack.put_to_frame(
                                module_name.to_string(),
                                Value::Handle(module_struct_handle),
                            );
                        } else {
                            // custom module
                            let mod_name = Path::new(&module_name)
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown");
                            let mod_bytecode = &self.bytecode
                                [self.pc + 1..(self.pc + (mod_bytecode_length as usize)) + 1];
                            self.pc += mod_bytecode_length as usize;
                            // here we should generate a definition of the module
                            // and push it onto the heap and add a Handle to the stack
                            // --
                            let exec_result = self.run_module(
                                &mod_name.to_string(),
                                mod_bytecode.to_vec(),
                                debug,
                            );
                            if exec_result.error.is_some() {
                                return exec_result;
                            }

                            // if members exported, add them to the scope
                            if let Some(result) = exec_result.result {
                                if let Value::Handle(r) = result {
                                    self.call_stack
                                        .put_to_frame(mod_name.to_string(), Value::Handle(r));
                                }
                            }
                        }
                    } else {
                        // TODO: use self-vm errors system
                        panic!("invalid value type as module name for import")
                    }

                    self.pc += 1;
                }
                Opcode::Export => {
                    let arg_ref = self.get_stack_values(&1)[0].clone();
                    if let Value::Handle(r) = arg_ref.clone() {
                        let arg = self.memory.resolve(&r);
                        if let MemObject::String(s) = arg {
                            if debug {
                                println!("EXPORT -> {}", s)
                            }
                            self.call_stack.add_export(s.to_string());
                        } else {
                            return VMExecutionResult::terminate_with_errors(
                                VMErrorType::ExportInvalidMemberType,
                                self,
                            );
                        }
                    } else {
                        return VMExecutionResult::terminate_with_errors(
                            VMErrorType::ExportInvalidMemberType,
                            self,
                        );
                    }
                    self.pc += 1;
                }
                Opcode::Return => {
                    let return_value = self.get_stack_values(&1)[0].clone();
                    return VMExecutionResult::terminate(Some(return_value));
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
                        return VMExecutionResult::terminate_with_errors(err, self);
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
                        return VMExecutionResult::terminate_with_errors(err, self);
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
                        return VMExecutionResult::terminate_with_errors(err, self);
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
                        return VMExecutionResult::terminate_with_errors(err, self);
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
                        return VMExecutionResult::terminate_with_errors(err, self);
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
                        return VMExecutionResult::terminate_with_errors(err, self);
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
                        return VMExecutionResult::terminate_with_errors(err, self);
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
                        return VMExecutionResult::terminate_with_errors(err, self);
                    }

                    self.pc += 1;
                }
                Opcode::FFI_Call => {
                    self.pc += 1; // consume call opcode
                    let args = self.get_function_call_args();
                    let mut resolved_args = Vec::new();
                    for val in args {
                        match self.value_to_string(val) {
                            Ok(v) => resolved_args.push(v),
                            Err(e) => return VMExecutionResult::terminate_with_errors(e, self),
                        }
                    }
                    if debug {
                        println!("CALL -> {}", resolved_args[0].to_string())
                    }
                    call_handler(&self.ffi_handlers, resolved_args);
                }
                _ => {
                    println!("unhandled opcode");
                    self.pc += 1;
                }
            };
        }

        VMExecutionResult::terminate(None)
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
            (Value::Handle(l), Value::Handle(r)) => {
                // here implement binary operations between different
                // types once the Handle is resolved to the actual value
                let l_heap_object = self.memory.resolve(&l);
                let r_heap_object = self.memory.resolve(&r);

                let result_value = match (l_heap_object, r_heap_object) {
                    (MemObject::String(left_string), MemObject::String(right_string)) => {
                        match operator {
                            "+" => {
                                let result_string = format!("{left_string}{right_string}");
                                self.memory.alloc(MemObject::String(result_string))
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

                value = Value::Handle(result_value);
            }
            (Value::Handle(_), Value::RawValue(_)) => {
                return Some(VMErrorType::TypeCoercionError(right))
            }
            (Value::RawValue(_), Value::Handle(_)) => {
                return Some(VMErrorType::TypeCoercionError(right))
            }
            _ => {
                panic!("invalid Value type for a binary expression")
            }
        }

        self.push_to_stack(value, None);
        None
    }

    fn run_module(
        &mut self,
        mod_name: &String,
        mod_bytecode: Vec<u8>,
        debug: bool,
    ) -> VMExecutionResult {
        let return_pc = self.pc;
        let main_bytecode = std::mem::take(&mut self.bytecode);

        self.call_stack.push();
        self.bytecode = mod_bytecode.clone();
        self.pc = 0;
        let mut mod_exec_result = self.run_bytecode(debug);

        // recover state after execution
        let mod_frame = self.call_stack.pop(); // here we should lookup the exports and store on a struct, then, return that struct on the VMExecutionResult
        if let Some(mut frame) = mod_frame {
            let exported_members = frame.get_exports();
            let exports_struct = StructLiteral::new(mod_name.to_string(), exported_members);
            let exports_handle = self.memory.alloc(MemObject::StructLiteral(exports_struct));

            mod_exec_result.result = Some(Value::Handle(exports_handle));
        }
        self.pc = return_pc;
        self.bytecode = main_bytecode;

        mod_exec_result
    }

    pub fn run_function(
        &mut self,
        func: &Function,
        caller: Option<Handle>,
        args: Vec<Value>,
        debug: bool,
    ) -> VMExecutionResult {
        let execution_result = match &func.engine {
            Engine::Bytecode(bytecode) => {
                let return_pc = self.pc;
                let main_bytecode = std::mem::take(&mut self.bytecode);

                self.call_stack.push();
                for (index, param) in func.parameters.iter().enumerate() {
                    if index < args.len() {
                        self.call_stack
                            .put_to_frame(param.clone(), args[index].clone());
                    } else {
                        self.call_stack
                            .put_to_frame(param.clone(), Value::RawValue(RawValue::Nothing));
                    }
                }
                self.bytecode = bytecode.clone();
                self.pc = 0;

                let function_exec_result = self.run_bytecode(debug);
                // recover state after execution
                self.call_stack.pop();
                self.pc = return_pc;
                self.bytecode = main_bytecode;

                function_exec_result
            }
            Engine::Native(native) => {
                if args.len() < func.parameters.len() {
                    // TODO: use self-vm errors system
                    panic!(
                        "function '{}' requires {} parameters, provided {}",
                        func.identifier,
                        func.parameters.len(),
                        args.len()
                    )
                }
                let execution_result = native(self, caller, args, debug);
                if let Ok(result) = execution_result {
                    // we could return the result value, using
                    // it as the return value of the function
                    VMExecutionResult {
                        error: None,
                        result: Some(result),
                    }
                } else {
                    VMExecutionResult {
                        error: Some(execution_result.unwrap_err()),
                        result: None,
                    }
                }
            }
        };

        return execution_result;
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
            }
            DataType::StructLiteral => 4, // fields count
            DataType::Vector => 4,        // elements count
            _ => {
                println!("data_type: {:#?}", data_type);
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

                let value_handle = self.memory.alloc(MemObject::String(value));
                Value::Handle(value_handle)
            }
            DataType::Vector => {
                let elements_count_bytes = if value.len() >= 4 {
                    &value[value.len() - 4..]
                } else {
                    panic!("Struct literal must contain more than 4 bytes");
                };

                let elements_count = u32::from_le_bytes(
                    elements_count_bytes
                        .try_into()
                        .expect("Provided value is incorrect"),
                );
                let elements = self.get_stack_values(&elements_count);

                let mut vector = Vector::new(elements);
                vector::init_vector_members(&mut vector, &self);
                printable_value = vector.to_string(self);

                let value_handle = self.memory.alloc(MemObject::Vector(vector));
                Value::Handle(value_handle)
            }
            DataType::StructLiteral => {
                let struct_type = self.get_stack_values(&1)[0].clone();
                let fields_count_bytes = if value.len() >= 4 {
                    &value[value.len() - 4..]
                } else {
                    panic!("Struct literal must contain more than 4 bytes");
                };

                let fields_count = u32::from_le_bytes(
                    fields_count_bytes
                        .try_into()
                        .expect("Provided value is incorrect"),
                );

                // we made *2 because, we're storing the field_value and the field_name
                let mut fields: HashMap<String, Value> = HashMap::new();
                let flat_fields = self.get_stack_values(&(fields_count * 2));
                for i in (0..fields_count * 2).step_by(2) {
                    let field_name_handle = flat_fields[i as usize].clone();
                    let field_value = flat_fields[(i + 1) as usize].clone();

                    // this is because we're using the existent infra for utf8 values
                    // and they are a heap allocated value, but there is also infra to
                    // storing strings in the stack and not in the heap
                    if let Value::Handle(field_handle) = field_name_handle {
                        let field_name = self.memory.free(field_handle);
                        if let MemObject::String(field_name) = field_name {
                            // add field with it's value to StructLiteral fields
                            fields.insert(field_name, field_value);
                        } else {
                            // TODO: handle with self-vm errors system
                            panic!("struct field_name must be a MemObject of type string");
                        }
                    } else {
                        // TODO: handle with self-vm errors system
                        panic!("struct field_name must be a Handle of a string");
                    };
                }

                let resolved_struct_type = match struct_type {
                    Value::Handle(v) => self.memory.resolve(&v),
                    Value::BoundAccess(_) => {
                        panic!("TODO: implement structs from bound acceses ")
                    }
                    _ => {
                        // TODO: use self-vm errors system
                        panic!("invalid struct type")
                    }
                };
                printable_value = resolved_struct_type.to_string(self);

                // here we should check if the struct exists and the each field
                // before allocating it in the heap
                let struct_literal =
                    StructLiteral::new(resolved_struct_type.to_string(self), fields);
                let value_handle = self.memory.alloc(MemObject::StructLiteral(struct_literal));
                Value::Handle(value_handle)
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
        Ok(value.to_string(self))
    }

    fn values_to_string(&mut self, args: Vec<Value>) -> Result<Vec<String>, VMErrorType> {
        let mut resolved_args = Vec::new();
        for val in args {
            match self.value_to_string(val) {
                Ok(v) => resolved_args.push(v),
                Err(e) => return Err(e),
            }
        }

        Ok(resolved_args)
    }

    pub fn read_offset(bytes: &[u8]) -> i32 {
        // TODO: use self-vm errors
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
                None => {
                    panic!("Stack underflow: trying to get '{num_of_values}' values from the stack")
                }
            }
        }

        args.reverse(); // invocation order
        args
    }

    pub fn get_handler(&self, handler: &str) -> Option<Handle> {
        self.handlers.get(handler).cloned()
    }
    pub fn push_to_stack(&mut self, value: Value, origin: Option<String>) {
        self.operand_stack
            .push(OperandsStackValue { value, origin });
    }

    pub fn debug_bytecode(&mut self) {
        println!("\n--- BYTECODE ----------\n");
        for (index, byte) in self.bytecode.iter().enumerate() {
            println!("[{index}] {}", byte)
        }
        // -------
        // THIS CODE IS COMMENTED FOR THE REASON THAT
        // I DON'T KNOW HOW TO HANDLE THE BYTECODE
        // TRANSLATION WITHOUT AFFECTING THE CREATIVE
        // FL0W. SO FOR THE MOMENT WE'RE AVOIDING THE
        // PROBLEM BY COMMENTING IT.
        // ✱
        // -------
        // let mut pc = 0;
        // let mut target_pc = 0;

        // let string_offset = self.bytecode.len().to_string();
        // while pc < self.bytecode.len() {
        //     let index = (pc + 1).to_string();
        //     let mut counter = 0;
        //     let printable_index = string_offset
        //         .chars()
        //         .map(|_| {
        //             let mut result = "".to_string();
        //             if let Some(char) = index.chars().nth(counter) {
        //                 result = char.to_string();
        //             } else {
        //                 result = " ".to_string();
        //             }
        //             counter += 1;
        //             return result;
        //         })
        //         .collect::<String>();

        //     if pc >= target_pc {
        //         // print instruction
        //         let (instruction, offset) = Translator::get_instruction(pc, &self.bytecode);
        //         let raw_instruction = format!("{}|    {:#?}", printable_index, self.bytecode[pc]);
        //         println!("{} <---- {}", raw_instruction, instruction.get_type());

        //         let instruction_info = Translator::get_instruction_info(&instruction);
        //         if instruction_info.len() > 0 {
        //             println!("------------ \n{}\n------------", instruction_info);
        //         }
        //         // + 1  the normal iteration increment over the bytecode
        //         target_pc = pc + offset + 1;
        //     } else {
        //         // print bytecode index
        //         println!("{}|    {:#?}", printable_index, self.bytecode[pc]);
        //     }

        //     pc += 1;
        // }
        //println!("\n--- BYTECODE INSTRUCTIONS ----------\n");
        //println!("{:#?}", Translator::new(self.bytecode.clone()).translate());
    }
}
