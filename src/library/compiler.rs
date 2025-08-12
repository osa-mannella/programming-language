use crate::library::ast::{ASTNode, ASTProgram};
use crate::library::lexer::{Lexer, TokenValue};
use crate::library::modules::{LoadedModule, ModuleDefinition, ModuleFunction, ModuleRegistry};
use crate::library::parser::Parser;
use core::panic;
use std::collections::HashMap;
use std::fs::{self, read_to_string};
use std::path::Path;
use std::process::exit;

#[derive(Debug, Clone)]
pub struct BytecodeHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub flags: u16,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ConstantValue {
    String(String),
    Number(f64),
    Boolean(bool),
    FunctionRef(u16),
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub value: ConstantValue,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub arg_count: u8,
    pub local_count: u8,
    pub offset: u32,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name_index: u16,
    pub field_count: u8,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name_index: u16,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct BytecodeProgram {
    pub header: BytecodeHeader,
    pub constants: Vec<Constant>,
    pub functions: Vec<Function>,
    pub enums: Vec<EnumDef>,
    pub opcode_map: HashMap<String, u8>,
    pub instructions: Vec<u8>,
}

#[derive(Debug)]
struct CompileContext {
    function_map: HashMap<String, u16>,
    enum_map: HashMap<String, u16>,
    loaded_modules: HashMap<String, LoadedModule>,
    module_registry: ModuleRegistry,
    variable_map: HashMap<String, u16>,
    variable_counter: u16,
    compilation_failed: bool,
}

fn load_static_lib(bytecode: &mut BytecodeProgram, context: &mut CompileContext, lib: &str) {
    let contents = read_to_string(format!("src/static/{}", lib))
        .unwrap_or_else(|_| panic!("Failed to load static libraries"));

    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if parser.had_error {
        exit(1);
    }

    for node in &program.nodes {
        collect_declarations(bytecode, context, node);
    }
}

impl BytecodeProgram {
    pub fn new() -> Self {
        let mut opcode_map = HashMap::new();

        opcode_map.insert("load_const".to_string(), 0x01);
        opcode_map.insert("load_global".to_string(), 0x02);
        opcode_map.insert("store_global".to_string(), 0x03);
        opcode_map.insert("load_local".to_string(), 0x04);
        opcode_map.insert("store_local".to_string(), 0x05);

        opcode_map.insert("add".to_string(), 0x10);
        opcode_map.insert("sub".to_string(), 0x11);
        opcode_map.insert("mul".to_string(), 0x12);
        opcode_map.insert("div".to_string(), 0x13);
        opcode_map.insert("power".to_string(), 0x1C);
        opcode_map.insert("equal".to_string(), 0x14);
        opcode_map.insert("not_equal".to_string(), 0x17);
        opcode_map.insert("less".to_string(), 0x15);
        opcode_map.insert("greater".to_string(), 0x16);
        opcode_map.insert("less_equal".to_string(), 0x18);
        opcode_map.insert("greater_equal".to_string(), 0x19);
        opcode_map.insert("and".to_string(), 0x1A);
        opcode_map.insert("or".to_string(), 0x1B);

        opcode_map.insert("jump".to_string(), 0x20);
        opcode_map.insert("jump_if_false".to_string(), 0x21);
        opcode_map.insert("jump_if_true".to_string(), 0x22);

        opcode_map.insert("call".to_string(), 0x30);
        opcode_map.insert("call_global".to_string(), 0x31);
        opcode_map.insert("return".to_string(), 0x32);
        opcode_map.insert("call_native".to_string(), 0x33);
        opcode_map.insert("call_indirect".to_string(), 0x34);

        opcode_map.insert("pop".to_string(), 0x40);
        opcode_map.insert("dup".to_string(), 0x41);

        opcode_map.insert("store_var".to_string(), 0x50);
        opcode_map.insert("load_var".to_string(), 0x51);

        opcode_map.insert("create_enum".to_string(), 0x52);
        opcode_map.insert("get_enum_variant".to_string(), 0x53);

        opcode_map.insert("match_literal".to_string(), 0x54);
        opcode_map.insert("match_enum_variant".to_string(), 0x55);
        opcode_map.insert("extract_enum_field".to_string(), 0x56);
        opcode_map.insert("match_fail".to_string(), 0x57);

        opcode_map.insert("string_concat".to_string(), 0x58);
        opcode_map.insert("index_access".to_string(), 0x59);
        opcode_map.insert("get_type".to_string(), 0x5A);
        opcode_map.insert("create_array".to_string(), 0x5B);
        opcode_map.insert("array_append".to_string(), 0x5C);
        opcode_map.insert("struct_create".to_string(), 0x5D);
        opcode_map.insert("match_struct_fields".to_string(), 0x5E);
        opcode_map.insert("extract_struct_field".to_string(), 0x5F);

        opcode_map.insert("halt".to_string(), 0xFF);

        Self {
            header: BytecodeHeader {
                magic: *b"MIRB",
                version: 1,
                flags: 0,
                message:
                    "Code flows like verse, swift and bright,\nBytecode dances in memory's light."
                        .to_string(),
            },
            constants: Vec::new(),
            functions: Vec::new(),
            enums: Vec::new(),
            opcode_map,
            instructions: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: ConstantValue) -> u16 {
        let index = self.constants.len() as u16;
        self.constants.push(Constant { value });
        index
    }

    pub fn add_function(&mut self, arg_count: u8, local_count: u8, offset: u32) -> u16 {
        let index = self.functions.len() as u16;
        self.functions.push(Function {
            arg_count,
            local_count,
            offset,
        });
        index
    }

    pub fn add_function_body(&mut self) {}

    pub fn add_enum(&mut self, name_index: u16, variants: Vec<EnumVariant>) -> u16 {
        let index = self.enums.len() as u16;
        self.enums.push(EnumDef {
            name_index,
            variants,
        });
        index
    }

    pub fn get_opcode(&self, name: &str) -> Option<u8> {
        self.opcode_map.get(name).copied()
    }

    pub fn emit_instruction(&mut self, opcode: u8) {
        self.instructions.push(opcode);
    }

    pub fn emit_instruction_u8(&mut self, opcode: u8, operand: u8) {
        self.instructions.push(opcode);
        self.instructions.push(operand);
    }

    pub fn emit_instruction_u16(&mut self, opcode: u8, operand: u16) {
        self.instructions.push(opcode);
        self.instructions.extend_from_slice(&operand.to_le_bytes());
    }

    pub fn emit_instruction_u8_u8(&mut self, opcode: u8, op1: u8, op2: u8) {
        self.instructions.push(opcode);
        self.instructions.push(op1);
        self.instructions.push(op2);
    }

    pub fn emit_instruction_u8_u8_u8(&mut self, opcode: u8, op1: u8, op2: u8, op3: u8) {
        self.instructions.push(opcode);
        self.instructions.push(op1);
        self.instructions.push(op2);
        self.instructions.push(op3);
    }

    pub fn current_offset(&self) -> u32 {
        self.instructions.len() as u32
    }

    pub fn update_function_offset(&mut self, func_index: u16, offset: u32) {
        if let Some(function) = self.functions.get_mut(func_index as usize) {
            function.offset = offset;
        }
    }
}

pub fn compile_program(ast: ASTProgram) -> Result<BytecodeProgram, String> {
    let mut bytecode = BytecodeProgram::new();
    let mut context = CompileContext {
        function_map: HashMap::new(),
        enum_map: HashMap::new(),
        loaded_modules: HashMap::new(),
        module_registry: ModuleRegistry::new(),
        variable_map: HashMap::new(),
        variable_counter: 0,
        compilation_failed: false,
    };

    load_static_lib(&mut bytecode, &mut context, "lib.mir");

    for node in &ast.nodes {
        collect_declarations(&mut bytecode, &mut context, node);
        if context.compilation_failed {
            return Err("Compilation failed during declaration collection".to_string());
        }
    }

    for node in &ast.nodes {
        collect_constants(&mut bytecode, &context, node);
    }

    for node in &ast.nodes {
        generate_instructions(&mut bytecode, &mut context, node);
        if context.compilation_failed {
            return Err("Compilation failed during code generation".to_string());
        }
    }

    if let Some(halt_opcode) = bytecode.get_opcode("halt") {
        bytecode.emit_instruction(halt_opcode);
    }

    Ok(bytecode)
}

fn collect_declarations(
    bytecode: &mut BytecodeProgram,
    context: &mut CompileContext,
    node: &ASTNode,
) {
    match node {
        ASTNode::FunctionStatement { name, params, body }
        | ASTNode::AsyncFunctionStatement { name, params, body } => {
            let func_index = bytecode.add_function(
                params.len() as u8,
                count_locals(body),
                0, // offset - will be calculated during instruction generation phase
            );
            match get_identifier_string(&name.value) {
                Ok(func_name) => {
                    context.function_map.insert(func_name, func_index);
                }
                Err(err) => {
                    eprintln!("Compile error: {}", err);
                }
            }
        }

        ASTNode::EnumStatement {
            name,
            variant_names,
            field_names,
            ..
        } => {
            let enum_name_idx = match get_identifier_string(&name.value) {
                Ok(enum_name) => bytecode.add_constant(ConstantValue::String(enum_name.clone())),
                Err(err) => {
                    eprintln!("Compile error: {}", err);
                    return;
                }
            };

            let mut variants = Vec::new();
            for (variant_name, field_list) in variant_names.iter().zip(field_names.iter()) {
                let variant_name_idx = match get_identifier_string(&variant_name.value) {
                    Ok(variant_name) => bytecode.add_constant(ConstantValue::String(variant_name)),
                    Err(err) => {
                        eprintln!("Compile error: {}", err);
                        continue;
                    }
                };
                variants.push(EnumVariant {
                    name_index: variant_name_idx,
                    field_count: field_list.len() as u8,
                });
            }

            let enum_index = bytecode.add_enum(enum_name_idx, variants);
            match get_identifier_string(&name.value) {
                Ok(enum_name) => {
                    context.enum_map.insert(enum_name, enum_index);
                }
                Err(err) => {
                    eprintln!("Compile error: {}", err);
                }
            }
        }

        ASTNode::IfExpression {
            then_branch,
            else_branch,
            ..
        } => {
            for stmt in then_branch {
                collect_declarations(bytecode, context, stmt);
            }
            if let Some(else_stmts) = else_branch {
                for stmt in else_stmts {
                    collect_declarations(bytecode, context, stmt);
                }
            }
        }

        ASTNode::MatchStatement { arms, .. } => {
            for arm in arms {
                for pattern in &arm.patterns {
                    collect_declarations(bytecode, context, pattern);
                }
                for expression in &arm.expression {
                    collect_declarations(bytecode, context, expression);
                }
            }
        }

        ASTNode::LambdaExpression { params, body }
        | ASTNode::AsyncLambdaExpression { params, body } => {
            bytecode.add_function(
                params.len() as u8,
                count_locals(body),
                0, // offset - will be calculated during instruction generation phase
            );

            for stmt in body {
                collect_declarations(bytecode, context, stmt);
            }
        }

        ASTNode::LetStatement {
            name: _,
            initializer,
        }
        | ASTNode::LetBangStatement {
            name: _,
            initializer,
        } => {
            match initializer.as_ref() {
                ASTNode::LambdaExpression { params, body }
                | ASTNode::AsyncLambdaExpression { params, body } => {
                    // Special handling for lambda assigned to variable
                    let _func_index = bytecode.add_function(
                        params.len() as u8,
                        count_locals(body),
                        0, // offset - will be calculated during instruction generation phase
                    );

                    // Don't add to function_map - lambda variables use indirect calls

                    // Recursively collect declarations from lambda body
                    for stmt in body {
                        collect_declarations(bytecode, context, stmt);
                    }
                }
                _ => {
                    // For non-lambda initializers, recursively collect declarations
                    collect_declarations(bytecode, context, initializer);
                }
            }
        }

        ASTNode::ImportStatement { path } => {
            let module_path = match &path.value {
                TokenValue::String(s) => s.clone(),
                TokenValue::Identifier(s) => s.clone(),
                _ => {
                    eprintln!("Compile error: Import path must be a string or identifier");
                    return;
                }
            };

            // Validate that the module exists before attempting to load
            if !validate_import_exists(&module_path, &context.module_registry) {
                eprintln!(
                    "Compile error (line {}): Module '{}' not found",
                    path.line, module_path
                );
                context.compilation_failed = true;
                return;
            }

            if !context.loaded_modules.contains_key(&module_path) {
                if is_local_file_import(&module_path) {
                    if let Some(loaded_module) = load_local_module(&module_path, bytecode) {
                        context
                            .loaded_modules
                            .insert(module_path.clone(), loaded_module);
                    } else {
                        eprintln!(
                            "Compile error (line {}): Could not load local module '{}'",
                            path.line, module_path
                        );
                        context.compilation_failed = true;
                        return;
                    }
                } else {
                    // Load from global registry
                    if let Some(loaded_module) =
                        context.module_registry.load_module(&module_path, bytecode)
                    {
                        context
                            .loaded_modules
                            .insert(module_path.clone(), loaded_module);
                    } else {
                        eprintln!(
                            "Compile error (line {}): Module '{}' not found in registry",
                            path.line, module_path
                        );
                        context.compilation_failed = true;
                        return;
                    }
                }
            }
        }

        _ => {}
    }
}

fn collect_constants(bytecode: &mut BytecodeProgram, context: &CompileContext, node: &ASTNode) {
    match node {
        ASTNode::Call { callee, .. } => {
            if let ASTNode::Variable { name } = callee.as_ref() {
                if let Ok(func_name) = get_identifier_string(&name.value) {
                    if let Some(&_func_index) = context.function_map.get(&func_name) {}
                }
            }
        }

        ASTNode::Literal { token } => match &token.value {
            TokenValue::String(s) => {
                bytecode.add_constant(ConstantValue::String(s.clone()));
            }
            TokenValue::Number(n) => {
                bytecode.add_constant(ConstantValue::Number(*n));
            }
            _ => {}
        },

        ASTNode::BoolLiteral { value } => {
            bytecode.add_constant(ConstantValue::Boolean(*value));
        }

        ASTNode::FunctionStatement { .. }
        | ASTNode::AsyncFunctionStatement { .. }
        | ASTNode::EnumStatement { .. } => {}

        ASTNode::IfExpression {
            then_branch,
            else_branch,
            ..
        } => {
            for stmt in then_branch {
                collect_constants(bytecode, context, stmt);
            }
            if let Some(else_stmts) = else_branch {
                for stmt in else_stmts {
                    collect_constants(bytecode, context, stmt);
                }
            }
        }

        ASTNode::MatchStatement { arms, .. } => {
            for arm in arms {
                for pattern in &arm.patterns {
                    collect_constants(bytecode, context, pattern);
                }
                for expression in &arm.expression {
                    collect_constants(bytecode, context, expression);
                }
            }
        }

        ASTNode::Binary { left, right, .. } => {
            collect_constants(bytecode, context, left);
            collect_constants(bytecode, context, right);
        }

        ASTNode::LambdaExpression { body, .. } => {
            for stmt in body {
                collect_constants(bytecode, context, stmt);
            }
        }

        ASTNode::AsyncLambdaExpression { body, .. } => {
            for stmt in body {
                collect_constants(bytecode, context, stmt);
            }
        }

        ASTNode::ExpressionStatement { expression } => {
            collect_constants(bytecode, context, expression);
        }

        ASTNode::LetStatement { initializer, .. }
        | ASTNode::LetBangStatement { initializer, .. } => {
            collect_constants(bytecode, context, initializer);
        }

        ASTNode::Grouping { expression } => {
            collect_constants(bytecode, context, expression);
        }

        ASTNode::PropertyAccess { object, .. } => {
            collect_constants(bytecode, context, object);
        }

        ASTNode::Pipeline { left, right } => {
            collect_constants(bytecode, context, left);
            collect_constants(bytecode, context, right);
        }

        ASTNode::ListLiteral { elements } => {
            for element in elements {
                collect_constants(bytecode, context, element);
            }
        }

        ASTNode::StructLiteral { values, .. } => {
            for value in values {
                collect_constants(bytecode, context, value);
            }
        }

        ASTNode::StructUpdate { base, values, .. } => {
            collect_constants(bytecode, context, base);
            for value in values {
                collect_constants(bytecode, context, value);
            }
        }

        ASTNode::EnumConstructor { values, .. } => {
            for value in values {
                collect_constants(bytecode, context, value);
            }
        }

        ASTNode::AwaitExpression { expression } => {
            collect_constants(bytecode, context, expression);
        }

        ASTNode::StringInterpolation { parts } => {
            for part in parts {
                collect_constants(bytecode, context, part);
            }
        }

        ASTNode::IndexAccess { object, index } => {
            collect_constants(bytecode, context, object);
            collect_constants(bytecode, context, index);
        }

        ASTNode::ArrayAppend { base, elements } => {
            collect_constants(bytecode, context, base);
            for element in elements {
                collect_constants(bytecode, context, element);
            }
        }

        ASTNode::ReturnStatement { expression } => {
            collect_constants(bytecode, context, expression);
        }

        ASTNode::Variable { .. }
        | ASTNode::ImportStatement { .. }
        | ASTNode::EnumDeconstructPattern { .. }
        | ASTNode::WildcardPattern
        | ASTNode::StructDeconstructPattern { .. } => {}
    }
}

fn count_locals(body: &[ASTNode]) -> u8 {
    let mut count = 0;
    for node in body {
        if matches!(
            node,
            ASTNode::LetStatement { .. } | ASTNode::LetBangStatement { .. }
        ) {
            count += 1;
        }
    }
    count
}

fn generate_instructions(
    bytecode: &mut BytecodeProgram,
    context: &mut CompileContext,
    node: &ASTNode,
) {
    generate_instructions_with_context(bytecode, context, node, false);
}

fn generate_instructions_with_context(
    bytecode: &mut BytecodeProgram,
    context: &mut CompileContext,
    node: &ASTNode,
    is_last_in_function: bool,
) {
    match node {
        ASTNode::Literal { token } => {
            let const_index = match &token.value {
                TokenValue::String(s) => {
                    find_or_add_constant(bytecode, ConstantValue::String(s.clone()))
                }
                TokenValue::Number(n) => find_or_add_constant(bytecode, ConstantValue::Number(*n)),
                _ => return,
            };

            if let Some(load_const_opcode) = bytecode.get_opcode("load_const") {
                bytecode.emit_instruction_u16(load_const_opcode, const_index);
            }
        }

        ASTNode::BoolLiteral { value } => {
            let const_index = find_or_add_constant(bytecode, ConstantValue::Boolean(*value));
            if let Some(load_const_opcode) = bytecode.get_opcode("load_const") {
                bytecode.emit_instruction_u16(load_const_opcode, const_index);
            }
        }

        ASTNode::Binary { left, right, op } => {
            generate_instructions_with_context(bytecode, context, left, false);
            generate_instructions_with_context(bytecode, context, right, false);

            let opcode = match op.kind {
                crate::library::lexer::TokenKind::Plus => bytecode.get_opcode("add"),
                crate::library::lexer::TokenKind::Minus => bytecode.get_opcode("sub"),
                crate::library::lexer::TokenKind::Star => bytecode.get_opcode("mul"),
                crate::library::lexer::TokenKind::Slash => bytecode.get_opcode("div"),
                crate::library::lexer::TokenKind::EqualEqual => bytecode.get_opcode("equal"),
                crate::library::lexer::TokenKind::BangEqual => bytecode.get_opcode("not_equal"),
                crate::library::lexer::TokenKind::Less => bytecode.get_opcode("less"),
                crate::library::lexer::TokenKind::Greater => bytecode.get_opcode("greater"),
                crate::library::lexer::TokenKind::LessEqual => bytecode.get_opcode("less_equal"),
                crate::library::lexer::TokenKind::GreaterEqual => {
                    bytecode.get_opcode("greater_equal")
                }
                crate::library::lexer::TokenKind::And => bytecode.get_opcode("and"),
                crate::library::lexer::TokenKind::Or => bytecode.get_opcode("or"),
                crate::library::lexer::TokenKind::Caret => bytecode.get_opcode("power"),
                _ => None,
            };

            if let Some(op) = opcode {
                bytecode.emit_instruction(op);
            }
        }

        ASTNode::Call { callee, arguments } => {
            for arg in arguments {
                generate_instructions_with_context(bytecode, context, arg, false);
            }

            match callee.as_ref() {
                ASTNode::Variable { name } => match get_identifier_string(&name.value) {
                    Ok(func_name) => {
                        if let Some(&func_index) = context.function_map.get(&func_name) {
                            // Direct function call - function is known at compile time
                            if let Some(call_opcode) = bytecode.get_opcode("call") {
                                bytecode.emit_instruction_u8_u8(
                                    call_opcode,
                                    func_index as u8,
                                    arguments.len() as u8,
                                );
                            }
                        } else {
                            // Indirect function call - variable contains a FunctionRef
                            // Load the variable containing the function reference
                            load_variable(bytecode, context, &func_name);

                            if let Some(call_indirect_opcode) = bytecode.get_opcode("call_indirect")
                            {
                                bytecode.emit_instruction_u8(
                                    call_indirect_opcode,
                                    arguments.len() as u8,
                                );
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Compile error: {}", err);
                    }
                },

                ASTNode::PropertyAccess { object, property } => {
                    if let ASTNode::Variable { name: module_name } = object.as_ref() {
                        let module_name_str = match get_identifier_string(&module_name.value) {
                            Ok(name) => name,
                            Err(err) => {
                                eprintln!("Compile error: {}", err);
                                context.compilation_failed = true;
                                return;
                            }
                        };
                        let function_name = match get_identifier_string(&property.value) {
                            Ok(name) => name,
                            Err(err) => {
                                eprintln!("Compile error: {}", err);
                                context.compilation_failed = true;
                                return;
                            }
                        };

                        if let Some(loaded_module) = context.loaded_modules.get(&module_name_str) {
                            if let Some(&func_index) =
                                loaded_module.function_indices.get(&function_name)
                            {
                                if let Some(module_func) = loaded_module
                                    .definition
                                    .functions
                                    .iter()
                                    .find(|f| f.name == function_name)
                                {
                                    if module_func.is_native {
                                        if let Some(call_native_opcode) =
                                            bytecode.get_opcode("call_native")
                                        {
                                            bytecode.emit_instruction_u8_u8(
                                                call_native_opcode,
                                                module_func.native_id.unwrap_or(0),
                                                arguments.len() as u8,
                                            );
                                        } else if let Some(call_global_opcode) =
                                            bytecode.get_opcode("call_global")
                                        {
                                            bytecode.emit_instruction_u8_u8(
                                                call_global_opcode,
                                                func_index as u8,
                                                arguments.len() as u8,
                                            );
                                        }
                                    } else {
                                        if let Some(call_opcode) = bytecode.get_opcode("call") {
                                            bytecode.emit_instruction_u8_u8(
                                                call_opcode,
                                                func_index as u8,
                                                arguments.len() as u8,
                                            );
                                        }
                                    }
                                }
                            }
                        } else {
                            eprintln!(
                                "Compile error: Module '{}' not found or function '{}' does not exist",
                                module_name_str, function_name
                            );
                            context.compilation_failed = true;
                            return;
                        }
                    } else {
                        // PropertyAccess where object is not a Variable (not a module call)
                        eprintln!(
                            "Compile error: Dot notation (.) can only be used for module function calls. Use bracket notation ['property'] for struct access instead."
                        );
                        context.compilation_failed = true;
                        return;
                    }
                }

                _ => {}
            }
        }

        ASTNode::Variable { name } => match get_identifier_string(&name.value) {
            Ok(var_name) => {
                if context.variable_map.contains_key(&var_name) {
                    load_variable(bytecode, context, &var_name);
                } else {
                    let const_index =
                        find_or_add_constant(bytecode, ConstantValue::String(var_name));
                    if let Some(load_global_opcode) = bytecode.get_opcode("load_global") {
                        bytecode.emit_instruction_u16(load_global_opcode, const_index);
                    }
                }
            }
            Err(err) => {
                eprintln!("Compile error: {}", err);
            }
        },

        ASTNode::ExpressionStatement { expression } => {
            generate_instructions_with_context(bytecode, context, expression, is_last_in_function);
            if !is_last_in_function {
                if let Some(pop_opcode) = bytecode.get_opcode("pop") {
                    bytecode.emit_instruction(pop_opcode);
                }
            }
        }

        ASTNode::FunctionStatement { name, body, .. }
        | ASTNode::AsyncFunctionStatement { name, body, .. } => {
            match get_identifier_string(&name.value) {
                Ok(func_name) => {
                    if let Some(&func_index) = context.function_map.get(&func_name) {
                        let old_var_map = context.variable_map.clone();
                        let old_var_counter = context.variable_counter;

                        let func_start_position =
                            if let Some(jump_opcode) = bytecode.get_opcode("jump") {
                                let end_jump = bytecode.current_offset();
                                bytecode.emit_instruction_u16(jump_opcode, 0);
                                end_jump
                            } else {
                                0
                            };
                        let offset = bytecode.current_offset();
                        bytecode.update_function_offset(func_index, offset);
                        for (i, stmt) in body.iter().enumerate() {
                            let is_last = i == body.len() - 1;
                            generate_instructions_with_context(bytecode, context, stmt, is_last);
                        }

                        if let Some(return_opcode) = bytecode.get_opcode("return") {
                            bytecode.emit_instruction(return_opcode);
                        }

                        patch_jump_offset(bytecode, func_start_position, bytecode.current_offset());

                        context.variable_map = old_var_map;
                        context.variable_counter = old_var_counter;
                    }
                }
                Err(err) => {
                    eprintln!("Compile error: {}", err);
                }
            }
        }

        ASTNode::Grouping { expression } => {
            generate_instructions_with_context(bytecode, context, expression, false);
        }

        ASTNode::IfExpression {
            condition,
            then_branch,
            else_branch,
        } => {
            generate_instructions_with_context(bytecode, context, condition, false);

            if let Some(jump_if_false_opcode) = bytecode.get_opcode("jump_if_false") {
                let false_jump_addr = bytecode.current_offset();
                bytecode.emit_instruction_u16(jump_if_false_opcode, 0);

                for stmt in then_branch {
                    generate_instructions_with_context(bytecode, context, stmt, false);
                }

                let end_jump_addr = if else_branch.is_some() {
                    if let Some(jump_opcode) = bytecode.get_opcode("jump") {
                        let addr = bytecode.current_offset();
                        bytecode.emit_instruction_u16(jump_opcode, 0);
                        Some(addr)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let else_start = bytecode.current_offset();
                patch_jump_offset(bytecode, false_jump_addr, else_start);

                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        generate_instructions_with_context(bytecode, context, stmt, false);
                    }
                }

                if let Some(jump_addr) = end_jump_addr {
                    let end_addr = bytecode.current_offset();
                    patch_jump_offset(bytecode, jump_addr, end_addr);
                }
            }
        }

        ASTNode::EnumStatement { .. } => {}

        ASTNode::EnumConstructor {
            enum_name,
            variant_name,
            field_names,
            values,
        } => {
            generate_enum_constructor(
                bytecode,
                context,
                enum_name,
                variant_name,
                field_names,
                values,
            );
        }

        ASTNode::LetStatement { name, initializer }
        | ASTNode::LetBangStatement { name, initializer } => {
            // TEMP MEASURE UNTIL STD LIB IS WRITTEN
            generate_let_statement(bytecode, context, name, initializer);
        }

        ASTNode::MatchStatement { value, arms } => {
            generate_match_statement(bytecode, context, value, arms);
        }

        ASTNode::StringInterpolation { parts } => {
            // Generate instructions to load each part onto the stack
            for part in parts {
                generate_instructions_with_context(bytecode, context, part, false);
            }

            // Generate string concatenation instructions
            if parts.len() > 1 {
                if let Some(concat_opcode) = bytecode.get_opcode("string_concat") {
                    bytecode.emit_instruction_u8(concat_opcode, parts.len() as u8);
                }
            }
        }

        ASTNode::IndexAccess { object, index } => {
            // Generate instructions to load the object and index onto the stack
            generate_instructions_with_context(bytecode, context, object, false);
            generate_instructions_with_context(bytecode, context, index, false);

            // Generate index access instruction with runtime type checking
            if let Some(index_access_opcode) = bytecode.get_opcode("index_access") {
                bytecode.emit_instruction(index_access_opcode);
            }
        }

        ASTNode::ArrayAppend { base, elements } => {
            // Generate instructions for array append operation
            generate_instructions_with_context(bytecode, context, base, false);
            for element in elements {
                generate_instructions_with_context(bytecode, context, element, false);
            }
            if let Some(array_append_opcode) = bytecode.get_opcode("array_append") {
                bytecode.emit_instruction_u8(array_append_opcode, elements.len() as u8);
            }
        }

        ASTNode::ListLiteral { elements } => {
            // Generate instructions to load each element onto the stack
            for element in elements {
                generate_instructions_with_context(bytecode, context, element, false);
            }
            // Generate array creation instruction with element count
            if let Some(create_array_opcode) = bytecode.get_opcode("create_array") {
                bytecode.emit_instruction_u8(create_array_opcode, elements.len() as u8);
            }
        }

        ASTNode::StructLiteral { keys, values } => {
            // Push field name then value onto the stack for each field
            for (key, value) in keys.iter().zip(values.iter()) {
                // Push field name as string constant
                let field_name_str = match get_identifier_string(&key.value) {
                    Ok(name) => name,
                    Err(err) => {
                        eprintln!("Compile error: {}", err);
                        continue;
                    }
                };
                let field_name_const =
                    find_or_add_constant(bytecode, ConstantValue::String(field_name_str));
                if let Some(load_const_opcode) = bytecode.get_opcode("load_const") {
                    bytecode.emit_instruction_u16(load_const_opcode, field_name_const);
                }

                // Push field value
                generate_instructions_with_context(bytecode, context, value, false);
            }

            // Generate struct creation instruction with field count
            if let Some(struct_create_opcode) = bytecode.get_opcode("struct_create") {
                bytecode.emit_instruction_u8(struct_create_opcode, keys.len() as u8);
            }
        }

        ASTNode::LambdaExpression { params, body }
        | ASTNode::AsyncLambdaExpression { params, body } => {
            let current_offset = bytecode.current_offset();

            // Look for an uncompiled function with matching signature
            let func_index = bytecode
                .functions
                .iter()
                .enumerate()
                .find(|(_, func)| func.arg_count == params.len() as u8 && func.offset == 0)
                .map(|(idx, _)| idx as u16);

            if let Some(func_idx) = func_index {
                // Set the function's offset to current position
                bytecode.update_function_offset(func_idx, current_offset);

                // Save current variable context
                let old_var_map = context.variable_map.clone();
                let old_var_counter = context.variable_counter;

                // Set up lambda parameters as local variables (0, 1, 2, ...)
                for (i, param) in params.iter().enumerate() {
                    if let Ok(param_name) = get_identifier_string(&param.value) {
                        context.variable_map.insert(param_name, i as u16);
                    }
                }
                context.variable_counter = params.len() as u16;

                // Generate lambda body instructions
                for (i, stmt) in body.iter().enumerate() {
                    let is_last = i == body.len() - 1;
                    generate_instructions_with_context(bytecode, context, stmt, is_last);
                }

                // Add return instruction
                if let Some(return_opcode) = bytecode.get_opcode("return") {
                    bytecode.emit_instruction(return_opcode);
                }

                // Restore variable context
                context.variable_map = old_var_map;
                context.variable_counter = old_var_counter;

                // Load function reference onto stack (for assignment to variable)
                if let Some(load_const_opcode) = bytecode.get_opcode("load_const") {
                    let func_const =
                        find_or_add_constant(bytecode, ConstantValue::FunctionRef(func_idx));
                    bytecode.emit_instruction_u16(load_const_opcode, func_const);
                }
            } else {
                eprintln!("Compile error: Could not find function index for lambda");
                context.compilation_failed = true;
            }
        }

        ASTNode::PropertyAccess { .. } => {
            // PropertyAccess should only be used for module function calls within Call nodes
            // Standalone property access (like obj.prop as an expression) is not allowed
            eprintln!(
                "Compile error: Dot notation (.) is only allowed for module function calls, not for property access. Use bracket notation ['property'] for struct access instead."
            );
            context.compilation_failed = true;
        }

        ASTNode::ReturnStatement { expression } => {
            generate_instructions_with_context(bytecode, context, expression, false);
            if let Some(return_opcode) = bytecode.get_opcode("return") {
                bytecode.emit_instruction(return_opcode);
            }
        }

        _ => {}
    }
}

fn find_or_add_constant(bytecode: &mut BytecodeProgram, value: ConstantValue) -> u16 {
    for (i, constant) in bytecode.constants.iter().enumerate() {
        match (&constant.value, &value) {
            (ConstantValue::String(a), ConstantValue::String(b)) if a == b => return i as u16,
            (ConstantValue::Number(a), ConstantValue::Number(b))
                if (a - b).abs() < f64::EPSILON =>
            {
                return i as u16;
            }
            (ConstantValue::Boolean(a), ConstantValue::Boolean(b)) if a == b => return i as u16,
            (ConstantValue::FunctionRef(a), ConstantValue::FunctionRef(b)) if a == b => {
                return i as u16;
            }
            _ => continue,
        }
    }

    bytecode.add_constant(value)
}

fn patch_jump_offset(bytecode: &mut BytecodeProgram, jump_addr: u32, target_addr: u32) {
    let offset = target_addr - jump_addr;
    let offset_bytes = (offset as u16).to_le_bytes();

    if let Some(byte1) = bytecode.instructions.get_mut((jump_addr + 1) as usize) {
        *byte1 = offset_bytes[0];
    }
    if let Some(byte2) = bytecode.instructions.get_mut((jump_addr + 2) as usize) {
        *byte2 = offset_bytes[1];
    }
}

fn get_identifier_string(value: &TokenValue) -> Result<String, String> {
    match value {
        TokenValue::Identifier(s) => Ok(s.clone()),
        _ => Err("Expected identifier token for variable name".to_string()),
    }
}

/// Generate bytecode for let statements
fn generate_let_statement(
    bytecode: &mut BytecodeProgram,
    context: &mut CompileContext,
    name: &crate::library::lexer::Token,
    initializer: &ASTNode,
) {
    match get_identifier_string(&name.value) {
        Ok(variable_name) => {
            generate_instructions_with_context(bytecode, context, initializer, false);
            store_variable(bytecode, context, &variable_name);
        }
        Err(err) => {
            eprintln!("Compile error: {}", err);
        }
    }
}

fn store_variable(bytecode: &mut BytecodeProgram, context: &mut CompileContext, var_name: &str) {
    let var_index = if let Some(&index) = context.variable_map.get(var_name) {
        index
    } else {
        let index = context.variable_counter;
        context.variable_map.insert(var_name.to_string(), index);
        context.variable_counter += 1;
        index
    };
    if let Some(store_opcode) = bytecode.get_opcode("store_var") {
        bytecode.emit_instruction_u16(store_opcode, var_index);
    }
}

fn load_variable(bytecode: &mut BytecodeProgram, context: &CompileContext, var_name: &str) {
    if let Some(&var_index) = context.variable_map.get(var_name) {
        if let Some(load_opcode) = bytecode.get_opcode("load_var") {
            bytecode.emit_instruction_u16(load_opcode, var_index);
        }
    } else {
        eprintln!("Compile error: Variable '{}' not found", var_name);
    }
}

fn generate_enum_constructor(
    bytecode: &mut BytecodeProgram,
    context: &mut CompileContext,
    enum_name: &crate::library::lexer::Token,
    variant_name: &crate::library::lexer::Token,
    field_names: &[crate::library::lexer::Token],
    values: &[ASTNode],
) {
    let enum_name_str = match get_identifier_string(&enum_name.value) {
        Ok(name) => name,
        Err(err) => {
            eprintln!("Compile error: {}", err);
            return;
        }
    };
    let variant_name_str = match get_identifier_string(&variant_name.value) {
        Ok(name) => name,
        Err(err) => {
            eprintln!("Compile error: {}", err);
            return;
        }
    };

    if let Some(&enum_index) = context.enum_map.get(&enum_name_str) {
        for value in values {
            generate_instructions_with_context(bytecode, context, value, false);
        }
        if let Some(enum_def) = bytecode.enums.get(enum_index as usize) {
            let mut variant_index = None;
            for (i, variant) in enum_def.variants.iter().enumerate() {
                if let Some(variant_const) = bytecode.constants.get(variant.name_index as usize) {
                    if let ConstantValue::String(name) = &variant_const.value {
                        if name == &variant_name_str {
                            variant_index = Some(i as u8);
                            break;
                        }
                    }
                }
            }

            if let Some(variant_idx) = variant_index {
                if values.len() != field_names.len() {
                    eprintln!(
                        "Compile error: Enum constructor field count mismatch. Expected {}, got {}",
                        field_names.len(),
                        values.len()
                    );
                    return;
                }
                if let Some(create_enum_opcode) = bytecode.get_opcode("create_enum") {
                    bytecode.emit_instruction_u8_u8_u8(
                        create_enum_opcode,
                        enum_index as u8,
                        variant_idx,
                        values.len() as u8,
                    );
                }
            } else {
                eprintln!(
                    "Compile error: Variant '{}' not found in enum '{}'",
                    variant_name_str, enum_name_str
                );
            }
        } else {
            eprintln!(
                "Compile error: Enum definition not found for index {}",
                enum_index
            );
        }
    } else {
        eprintln!("Compile error: Enum '{}' not found", enum_name_str);
    }
}

fn validate_import_exists(module_path: &str, module_registry: &ModuleRegistry) -> bool {
    if is_local_file_import(module_path) {
        resolve_module_path(module_path).is_some()
    } else {
        module_registry.module_exists(module_path)
    }
}

fn is_local_file_import(module_path: &str) -> bool {
    module_path.contains("/")
        || module_path.contains("\\")
        || module_path.starts_with("./")
        || module_path.starts_with("../")
        || module_path.ends_with(".mir")
        || Path::new(module_path).exists()
}

fn load_local_module(module_path: &str, bytecode: &mut BytecodeProgram) -> Option<LoadedModule> {
    let file_path = resolve_module_path(module_path)?;
    let source = fs::read_to_string(&file_path).ok()?;
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program();

    if parser.had_error {
        eprintln!("Parse errors in module: {}", file_path);
        return None;
    }

    let module_def = extract_module_definition(&ast, module_path);
    let mut loaded_module = LoadedModule {
        definition: module_def.clone(),
        function_indices: HashMap::new(),
        constant_indices: HashMap::new(),
    };
    for (i, constant) in module_def.constants.iter().enumerate() {
        let const_name = format!("{}._const_{}", module_path, i);
        let const_index = bytecode.add_constant(constant.clone());
        loaded_module
            .constant_indices
            .insert(const_name, const_index);
    }
    for func in &module_def.functions {
        let func_index = bytecode.add_function(func.arg_count, 0, 0); // Local count will be determined later
        loaded_module
            .function_indices
            .insert(func.name.clone(), func_index);

        let func_name_const = format!("{}.{}", module_path, func.name);
        bytecode.add_constant(ConstantValue::String(func_name_const));
    }

    Some(loaded_module)
}

fn resolve_module_path(module_path: &str) -> Option<String> {
    if module_path.ends_with(".mir") {
        if Path::new(module_path).exists() {
            return Some(module_path.to_string());
        }
    }
    let with_extension = format!("{}.mir", module_path);
    if Path::new(&with_extension).exists() {
        return Some(with_extension);
    }
    let relative_paths = vec![
        format!("./{}", module_path),
        format!("./{}.mir", module_path),
        format!("./modules/{}", module_path),
        format!("./modules/{}.mir", module_path),
    ];

    for path in relative_paths {
        if Path::new(&path).exists() {
            return Some(path);
        }
    }

    None
}

fn extract_module_definition(ast: &ASTProgram, module_name: &str) -> ModuleDefinition {
    let mut functions = Vec::new();
    let constants = Vec::new();
    for node in &ast.nodes {
        match node {
            ASTNode::FunctionStatement { name, params, .. }
            | ASTNode::AsyncFunctionStatement { name, params, .. } => {
                match get_identifier_string(&name.value) {
                    Ok(func_name) => {
                        functions.push(ModuleFunction {
                            name: func_name,
                            arg_count: params.len() as u8,
                            is_native: false,
                            native_id: None,
                        });
                    }
                    Err(err) => {
                        eprintln!("Compile error: {}", err);
                    }
                }
            }
            _ => {}
        }
    }

    ModuleDefinition {
        name: module_name.to_string(),
        functions,
        constants,
    }
}

fn generate_match_statement(
    bytecode: &mut BytecodeProgram,
    context: &mut CompileContext,
    value: &ASTNode,
    arms: &[super::ast::MatchArm],
) {
    validate_match_arms_field_consistency(context, arms);
    generate_instructions_with_context(bytecode, context, value, false);

    let mut match_end_jumps: Vec<u32> = Vec::new();
    for arm in arms.iter() {
        let mut arm_success_jumps: Vec<u32> = Vec::new();
        for pattern in arm.patterns.iter() {
            if let Some(dup_opcode) = bytecode.get_opcode("dup") {
                bytecode.emit_instruction(dup_opcode);
            }

            generate_pattern_match(bytecode, context, pattern);

            if let Some(jump_if_true_opcode) = bytecode.get_opcode("jump_if_true") {
                let jump_to_arm_body = bytecode.current_offset();
                bytecode.emit_instruction_u16(jump_if_true_opcode, 0);
                arm_success_jumps.push(jump_to_arm_body);
            }
        }

        let skip_to_next_arm = if let Some(jump_opcode) = bytecode.get_opcode("jump") {
            let jump_addr = bytecode.current_offset();
            bytecode.emit_instruction_u16(jump_opcode, 0);
            Some(jump_addr)
        } else {
            None
        };

        let arm_body_start = bytecode.current_offset();

        for jump_addr in arm_success_jumps {
            patch_jump_offset(bytecode, jump_addr, arm_body_start);
        }

        if let Some(pop_opcode) = bytecode.get_opcode("pop") {
            bytecode.emit_instruction(pop_opcode);
        }

        bind_pattern_variables(bytecode, context, &arm.patterns[0]);

        for expression in &arm.expression {
            generate_instructions_with_context(bytecode, context, expression, false);
        }

        if let Some(jump_opcode) = bytecode.get_opcode("jump") {
            let end_jump = bytecode.current_offset();
            bytecode.emit_instruction_u16(jump_opcode, 0);
            match_end_jumps.push(end_jump);
        }

        let next_arm_start = bytecode.current_offset();

        if let Some(skip_addr) = skip_to_next_arm {
            patch_jump_offset(bytecode, skip_addr, next_arm_start);
        }
    }

    if let Some(match_fail_opcode) = bytecode.get_opcode("match_fail") {
        bytecode.emit_instruction(match_fail_opcode);
    }

    let match_end = bytecode.current_offset();

    for jump_addr in match_end_jumps {
        patch_jump_offset(bytecode, jump_addr, match_end);
    }
}

fn generate_pattern_match(
    bytecode: &mut BytecodeProgram,
    context: &CompileContext,
    pattern: &ASTNode,
) {
    match pattern {
        ASTNode::Literal { token } => {
            let const_index = match &token.value {
                crate::library::lexer::TokenValue::String(s) => {
                    find_or_add_constant(bytecode, ConstantValue::String(s.clone()))
                }
                crate::library::lexer::TokenValue::Number(n) => {
                    find_or_add_constant(bytecode, ConstantValue::Number(*n))
                }
                _ => return,
            };

            if let Some(match_literal_opcode) = bytecode.get_opcode("match_literal") {
                bytecode.emit_instruction_u16(match_literal_opcode, const_index);
            }
        }

        ASTNode::BoolLiteral { value } => {
            let const_index = find_or_add_constant(bytecode, ConstantValue::Boolean(*value));
            if let Some(match_literal_opcode) = bytecode.get_opcode("match_literal") {
                bytecode.emit_instruction_u16(match_literal_opcode, const_index);
            }
        }
        ASTNode::EnumDeconstructPattern {
            enum_name,
            variant_name,
            field_names: _,
        } => {
            let enum_name_str = match get_identifier_string(&enum_name.value) {
                Ok(name) => name,
                Err(err) => {
                    eprintln!("Compile error: {}", err);
                    return;
                }
            };
            let variant_name_str = match get_identifier_string(&variant_name.value) {
                Ok(name) => name,
                Err(err) => {
                    eprintln!("Compile error: {}", err);
                    return;
                }
            };

            if let Some(&enum_index) = context.enum_map.get(&enum_name_str) {
                // Find variant index
                if let Some(enum_def) = bytecode.enums.get(enum_index as usize) {
                    let mut variant_index = None;
                    for (i, variant) in enum_def.variants.iter().enumerate() {
                        if let Some(variant_const) =
                            bytecode.constants.get(variant.name_index as usize)
                        {
                            if let ConstantValue::String(name) = &variant_const.value {
                                if name == &variant_name_str {
                                    variant_index = Some(i as u8);
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(variant_idx) = variant_index {
                        // Emit match_enum_variant instruction
                        if let Some(match_enum_opcode) = bytecode.get_opcode("match_enum_variant") {
                            bytecode.emit_instruction_u8_u8(
                                match_enum_opcode,
                                enum_index as u8,
                                variant_idx,
                            );
                        }
                    }
                }
            }
        }
        ASTNode::Variable { name: _ } => {
            let true_const = find_or_add_constant(bytecode, ConstantValue::Boolean(true));
            if let Some(load_const_opcode) = bytecode.get_opcode("load_const") {
                bytecode.emit_instruction_u16(load_const_opcode, true_const);
            }
        }

        ASTNode::WildcardPattern => {
            // Wildcard pattern always matches - push true onto the stack
            let true_const = find_or_add_constant(bytecode, ConstantValue::Boolean(true));
            if let Some(load_const_opcode) = bytecode.get_opcode("load_const") {
                bytecode.emit_instruction_u16(load_const_opcode, true_const);
            }
        }

        ASTNode::StructDeconstructPattern { field_names } => {
            // Generate field name constants and check if struct has all required fields
            let mut field_const_indices = Vec::new();
            for field_token in field_names {
                if let Ok(field_name) = get_identifier_string(&field_token.value) {
                    let field_const =
                        find_or_add_constant(bytecode, ConstantValue::String(field_name));
                    field_const_indices.push(field_const);
                } else {
                    eprintln!("Compile error: Invalid field name in struct pattern");
                    return;
                }
            }

            // Load field names as constants for runtime checking
            for field_const_idx in field_const_indices {
                if let Some(load_const_opcode) = bytecode.get_opcode("load_const") {
                    bytecode.emit_instruction_u16(load_const_opcode, field_const_idx);
                }
            }

            // Emit match_struct_fields instruction with field count
            if let Some(match_struct_opcode) = bytecode.get_opcode("match_struct_fields") {
                bytecode.emit_instruction_u8(match_struct_opcode, field_names.len() as u8);
            }
        }

        _ => {
            eprintln!("Compile error: Unsupported pattern type in match statement");
        }
    }
}

fn bind_pattern_variables(
    bytecode: &mut BytecodeProgram,
    context: &mut CompileContext,
    pattern: &ASTNode,
) {
    match pattern {
        ASTNode::EnumDeconstructPattern { field_names, .. } => {
            for (field_index, field_token) in field_names.iter().enumerate() {
                if let Ok(field_name) = get_identifier_string(&field_token.value) {
                    // Duplicate the enum value on stack
                    if let Some(dup_opcode) = bytecode.get_opcode("dup") {
                        bytecode.emit_instruction(dup_opcode);
                    }

                    // Extract the field value
                    if let Some(extract_field_opcode) = bytecode.get_opcode("extract_enum_field") {
                        bytecode.emit_instruction_u8(extract_field_opcode, field_index as u8);
                    }

                    // Store in local variable
                    store_variable(bytecode, context, &field_name);
                }
            }
            if let Some(pop_opcode) = bytecode.get_opcode("pop") {
                bytecode.emit_instruction(pop_opcode);
            }
        }

        ASTNode::Variable { name } => {
            if let Ok(var_name) = get_identifier_string(&name.value) {
                store_variable(bytecode, context, &var_name);
            }
        }

        ASTNode::WildcardPattern => {
            // Wildcard pattern doesn't bind any variables - just consume the value
            if let Some(pop_opcode) = bytecode.get_opcode("pop") {
                bytecode.emit_instruction(pop_opcode);
            }
        }

        ASTNode::StructDeconstructPattern { field_names } => {
            // Bind each field from the struct to a local variable
            for field_token in field_names {
                if let Ok(field_name) = get_identifier_string(&field_token.value) {
                    // Duplicate the struct value on stack
                    if let Some(dup_opcode) = bytecode.get_opcode("dup") {
                        bytecode.emit_instruction(dup_opcode);
                    }

                    // Load the field name as a constant
                    let field_name_const =
                        find_or_add_constant(bytecode, ConstantValue::String(field_name.clone()));
                    if let Some(load_const_opcode) = bytecode.get_opcode("load_const") {
                        bytecode.emit_instruction_u16(load_const_opcode, field_name_const);
                    }

                    // Extract the field value from the struct
                    if let Some(extract_field_opcode) = bytecode.get_opcode("extract_struct_field")
                    {
                        bytecode.emit_instruction(extract_field_opcode);
                    }

                    // Store the field value in a local variable with the same name as the field
                    store_variable(bytecode, context, &field_name);
                } else {
                    eprintln!("Compile error: Invalid field name in struct destructuring");
                }
            }

            // Clean up the original struct from the stack
            if let Some(pop_opcode) = bytecode.get_opcode("pop") {
                bytecode.emit_instruction(pop_opcode);
            }
        }

        _ => {}
    }
}

fn validate_match_arms_field_consistency(_context: &CompileContext, arms: &[super::ast::MatchArm]) {
    for arm in arms {
        if arm.patterns.len() > 1 {
            let mut expected_fields: Option<Vec<String>> = None;

            for pattern in &arm.patterns {
                if let ASTNode::EnumDeconstructPattern { field_names, .. } = pattern {
                    let current_fields: Vec<String> = field_names
                        .iter()
                        .filter_map(|token| get_identifier_string(&token.value).ok())
                        .collect();

                    if let Some(ref expected) = expected_fields {
                        if *expected != current_fields {
                            eprintln!(
                                "Compile error: Inconsistent field names in OR pattern. Expected {:?}, got {:?}",
                                expected, current_fields
                            );
                            return;
                        }
                    } else {
                        expected_fields = Some(current_fields);
                    }
                }
            }
        }
    }
}
