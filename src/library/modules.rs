use std::collections::HashMap;
use crate::library::compiler::{BytecodeProgram, ConstantValue};

#[derive(Debug, Clone)]
pub struct ModuleFunction {
    pub name: String,
    pub arg_count: u8,
    pub is_native: bool, // True for built-in functions, false for user-defined
    pub native_id: Option<u8>, // ID for native function lookup in VM
}

#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    pub name: String,
    pub functions: Vec<ModuleFunction>,
    pub constants: Vec<ConstantValue>,
}

#[derive(Debug, Clone)]
pub struct LoadedModule {
    pub definition: ModuleDefinition,
    pub function_indices: HashMap<String, u16>, // Maps function names to bytecode function indices
    pub constant_indices: HashMap<String, u16>, // Maps constant names to bytecode constant indices
}

#[derive(Debug)]
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleDefinition>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            modules: HashMap::new(),
        };
        
        // Register standard library modules
        registry.register_stdlib_modules();
        registry
    }

    pub fn register_module(&mut self, module: ModuleDefinition) {
        self.modules.insert(module.name.clone(), module);
    }

    pub fn get_module(&self, name: &str) -> Option<&ModuleDefinition> {
        self.modules.get(name)
    }

    pub fn load_module(
        &self,
        name: &str,
        bytecode: &mut BytecodeProgram,
    ) -> Option<LoadedModule> {
        let module_def = self.get_module(name)?.clone();
        let mut loaded_module = LoadedModule {
            definition: module_def.clone(),
            function_indices: HashMap::new(),
            constant_indices: HashMap::new(),
        };

        // Add module constants to bytecode
        for (i, constant) in module_def.constants.iter().enumerate() {
            let const_name = format!("{}._const_{}", name, i);
            let const_index = bytecode.add_constant(constant.clone());
            loaded_module.constant_indices.insert(const_name, const_index);
        }

        // Add module functions to bytecode
        for func in &module_def.functions {
            let func_index = if func.is_native {
                // For native functions, create a placeholder function entry
                // The VM will handle the actual implementation
                bytecode.add_function(func.arg_count, 0, 0)
            } else {
                // For user-defined functions, they would be compiled separately
                bytecode.add_function(func.arg_count, 0, 0)
            };

            loaded_module.function_indices.insert(func.name.clone(), func_index);
            
            // Add function name as constant for runtime lookup
            let func_name_const = format!("{}.{}", name, func.name);
            bytecode.add_constant(ConstantValue::String(func_name_const));
        }

        Some(loaded_module)
    }

    fn register_stdlib_modules(&mut self) {
        // IO Module
        let io_module = ModuleDefinition {
            name: "IO".to_string(),
            functions: vec![
                ModuleFunction {
                    name: "print".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x01), // Native function ID for VM
                },
                ModuleFunction {
                    name: "println".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x02),
                },
                ModuleFunction {
                    name: "input".to_string(),
                    arg_count: 0,
                    is_native: true,
                    native_id: Some(0x03),
                },
                ModuleFunction {
                    name: "read_file".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x04),
                },
                ModuleFunction {
                    name: "write_file".to_string(),
                    arg_count: 2,
                    is_native: true,
                    native_id: Some(0x05),
                },
            ],
            constants: vec![
                ConstantValue::String("stdout".to_string()),
                ConstantValue::String("stderr".to_string()),
                ConstantValue::String("stdin".to_string()),
            ],
        };

        // Math Module
        let math_module = ModuleDefinition {
            name: "Math".to_string(),
            functions: vec![
                ModuleFunction {
                    name: "sqrt".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x10),
                },
                ModuleFunction {
                    name: "pow".to_string(),
                    arg_count: 2,
                    is_native: true,
                    native_id: Some(0x11),
                },
                ModuleFunction {
                    name: "sin".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x12),
                },
                ModuleFunction {
                    name: "cos".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x13),
                },
                ModuleFunction {
                    name: "floor".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x14),
                },
                ModuleFunction {
                    name: "ceil".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x15),
                },
            ],
            constants: vec![
                ConstantValue::Number(std::f64::consts::PI),
                ConstantValue::Number(std::f64::consts::E),
            ],
        };

        // String Module
        let string_module = ModuleDefinition {
            name: "String".to_string(),
            functions: vec![
                ModuleFunction {
                    name: "length".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x20),
                },
                ModuleFunction {
                    name: "substring".to_string(),
                    arg_count: 3,
                    is_native: true,
                    native_id: Some(0x21),
                },
                ModuleFunction {
                    name: "concat".to_string(),
                    arg_count: 2,
                    is_native: true,
                    native_id: Some(0x22),
                },
                ModuleFunction {
                    name: "split".to_string(),
                    arg_count: 2,
                    is_native: true,
                    native_id: Some(0x23),
                },
                ModuleFunction {
                    name: "trim".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x24),
                },
            ],
            constants: vec![
                ConstantValue::String(" ".to_string()),
                ConstantValue::String("\n".to_string()),
                ConstantValue::String("\t".to_string()),
            ],
        };

        // List Module
        let list_module = ModuleDefinition {
            name: "List".to_string(),
            functions: vec![
                ModuleFunction {
                    name: "length".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x30),
                },
                ModuleFunction {
                    name: "push".to_string(),
                    arg_count: 2,
                    is_native: true,
                    native_id: Some(0x31),
                },
                ModuleFunction {
                    name: "pop".to_string(),
                    arg_count: 1,
                    is_native: true,
                    native_id: Some(0x32),
                },
                ModuleFunction {
                    name: "get".to_string(),
                    arg_count: 2,
                    is_native: true,
                    native_id: Some(0x33),
                },
                ModuleFunction {
                    name: "set".to_string(),
                    arg_count: 3,
                    is_native: true,
                    native_id: Some(0x34),
                },
                ModuleFunction {
                    name: "map".to_string(),
                    arg_count: 2,
                    is_native: true,
                    native_id: Some(0x35),
                },
                ModuleFunction {
                    name: "filter".to_string(),
                    arg_count: 2,
                    is_native: true,
                    native_id: Some(0x36),
                },
            ],
            constants: vec![],
        };

        self.register_module(io_module);
        self.register_module(math_module);
        self.register_module(string_module);
        self.register_module(list_module);
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}