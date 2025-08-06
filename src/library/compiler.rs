use crate::library::ast::{ASTNode, ASTProgram};
use crate::library::lexer::{Lexer, TokenValue};
use crate::library::modules::{LoadedModule, ModuleDefinition, ModuleFunction, ModuleRegistry};
use crate::library::parser::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
    Null,
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
    variable_map: HashMap<String, u16>, // Maps variable names to indices
    variable_counter: u16,              // Counter for assigning variable indices
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
        opcode_map.insert("equal".to_string(), 0x14);
        opcode_map.insert("less".to_string(), 0x15);
        opcode_map.insert("greater".to_string(), 0x16);

        opcode_map.insert("jump".to_string(), 0x20);
        opcode_map.insert("jump_if_false".to_string(), 0x21);

        opcode_map.insert("call".to_string(), 0x30);
        opcode_map.insert("call_global".to_string(), 0x31);
        opcode_map.insert("call_native".to_string(), 0x33);
        opcode_map.insert("return".to_string(), 0x32);

        opcode_map.insert("pop".to_string(), 0x40);
        opcode_map.insert("dup".to_string(), 0x41);

        // Variable operations
        opcode_map.insert("store_var".to_string(), 0x50);
        opcode_map.insert("load_var".to_string(), 0x51);

        // Enum operations
        opcode_map.insert("create_enum".to_string(), 0x52);
        opcode_map.insert("get_enum_variant".to_string(), 0x53);

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

pub fn compile_program(ast: ASTProgram) -> BytecodeProgram {
    let mut bytecode = BytecodeProgram::new();
    let mut context = CompileContext {
        function_map: HashMap::new(),
        enum_map: HashMap::new(),
        loaded_modules: HashMap::new(),
        module_registry: ModuleRegistry::new(),
        variable_map: HashMap::new(),
        variable_counter: 0,
    };

    for node in &ast.nodes {
        collect_declarations(&mut bytecode, &mut context, node);
    }

    for node in &ast.nodes {
        collect_constants(&mut bytecode, &context, node);
    }

    for node in &ast.nodes {
        generate_instructions(&mut bytecode, &mut context, node);
    }

    if let Some(halt_opcode) = bytecode.get_opcode("halt") {
        bytecode.emit_instruction(halt_opcode);
    }

    bytecode
}

fn collect_declarations(
    bytecode: &mut BytecodeProgram,
    context: &mut CompileContext,
    node: &ASTNode,
) {
    match node {
        ASTNode::FunctionStatement { name, params, body } => {
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

        ASTNode::AsyncFunctionStatement { name, params, body } => {
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
                collect_declarations(bytecode, context, &arm.expression);
            }
        }

        ASTNode::LambdaExpression { params, body } => {
            bytecode.add_function(
                params.len() as u8,
                count_locals(body),
                0, // offset - will be calculated during instruction generation phase
            );

            for stmt in body {
                collect_declarations(bytecode, context, stmt);
            }
        }

        ASTNode::AsyncLambdaExpression { params, body } => {
            bytecode.add_function(
                params.len() as u8,
                count_locals(body),
                0, // offset - will be calculated during instruction generation phase
            );

            for stmt in body {
                collect_declarations(bytecode, context, stmt);
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

            if !context.loaded_modules.contains_key(&module_path) {
                if is_local_file_import(&module_path) {
                    if let Some(loaded_module) = load_local_module(&module_path, bytecode) {
                        context
                            .loaded_modules
                            .insert(module_path.clone(), loaded_module);
                    } else {
                        eprintln!("Error: Could not load local module '{}'", module_path);
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
                        eprintln!("Warning: Module '{}' not found in registry", module_path);
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
                    if let Some(&_func_index) = context.function_map.get(&func_name) {
                    }
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
                collect_constants(bytecode, context, &arm.expression);
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

        ASTNode::Variable { .. }
        | ASTNode::ImportStatement { .. }
        | ASTNode::EnumDeconstructPattern { .. } => {}
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
            generate_instructions(bytecode, context, left);
            generate_instructions(bytecode, context, right);

            let opcode = match op.kind {
                crate::library::lexer::TokenKind::Plus => bytecode.get_opcode("add"),
                crate::library::lexer::TokenKind::Minus => bytecode.get_opcode("sub"),
                crate::library::lexer::TokenKind::Star => bytecode.get_opcode("mul"),
                crate::library::lexer::TokenKind::Slash => bytecode.get_opcode("div"),
                crate::library::lexer::TokenKind::EqualEqual => bytecode.get_opcode("equal"),
                crate::library::lexer::TokenKind::Less => bytecode.get_opcode("less"),
                crate::library::lexer::TokenKind::Greater => bytecode.get_opcode("greater"),
                _ => None,
            };

            if let Some(op) = opcode {
                bytecode.emit_instruction(op);
            }
        }

        ASTNode::Call { callee, arguments } => {
            for arg in arguments {
                generate_instructions(bytecode, context, arg);
            }

            match callee.as_ref() {
                ASTNode::Variable { name } => {
                    match get_identifier_string(&name.value) {
                        Ok(func_name) => {
                            if let Some(&func_index) = context.function_map.get(&func_name) {
                                if let Some(call_opcode) = bytecode.get_opcode("call") {
                                    bytecode.emit_instruction_u8_u8(
                                        call_opcode,
                                        func_index as u8,
                                        arguments.len() as u8,
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Compile error: {}", err);
                        }
                    }
                }

                ASTNode::PropertyAccess { object, property } => {
                    if let ASTNode::Variable { name: module_name } = object.as_ref() {
                        let module_name_str = match get_identifier_string(&module_name.value) {
                            Ok(name) => name,
                            Err(err) => {
                                eprintln!("Compile error: {}", err);
                                return;
                            }
                        };
                        let function_name = match get_identifier_string(&property.value) {
                            Ok(name) => name,
                            Err(err) => {
                                eprintln!("Compile error: {}", err);
                                return;
                            }
                        };

                        if let Some(loaded_module) = context.loaded_modules.get(&module_name_str) {
                            if let Some(&func_index) =
                                loaded_module.function_indices.get(&function_name)
                            {
                                // Check if this is a native function
                                if let Some(module_func) = loaded_module
                                    .definition
                                    .functions
                                    .iter()
                                    .find(|f| f.name == function_name)
                                {
                                    if module_func.is_native {
                                        // For native functions, use call_native opcode with native_id
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
                                            // Fallback to call_global if call_native not available
                                            bytecode.emit_instruction_u8_u8(
                                                call_global_opcode,
                                                func_index as u8,
                                                arguments.len() as u8,
                                            );
                                        }
                                    } else {
                                        // For user-defined functions, use regular call
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
                        }
                    }
                }

                _ => {}
            }
        }

        ASTNode::Variable { name } => {
            match get_identifier_string(&name.value) {
                Ok(var_name) => {
                    // Check if it's a local variable first
                    if context.variable_map.contains_key(&var_name) {
                        load_variable(bytecode, context, &var_name);
                    } else {
                        // Fallback to global variable loading
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
            }
        }

        ASTNode::ExpressionStatement { expression } => {
            generate_instructions(bytecode, context, expression);
            if let Some(pop_opcode) = bytecode.get_opcode("pop") {
                bytecode.emit_instruction(pop_opcode);
            }
        }

        ASTNode::FunctionStatement { name, body, .. } => {
            match get_identifier_string(&name.value) {
                Ok(func_name) => {
                    if let Some(&func_index) = context.function_map.get(&func_name) {
                        let offset = bytecode.current_offset();
                        bytecode.update_function_offset(func_index, offset);

                        // Create new variable scope for function
                        let old_var_map = context.variable_map.clone();
                        let old_var_counter = context.variable_counter;

                        for stmt in body {
                            generate_instructions(bytecode, context, stmt);
                        }

                        if let Some(return_opcode) = bytecode.get_opcode("return") {
                            bytecode.emit_instruction(return_opcode);
                        }

                        // Restore previous scopes
                        context.variable_map = old_var_map;
                        context.variable_counter = old_var_counter;
                    }
                }
                Err(err) => {
                    eprintln!("Compile error: {}", err);
                }
            }
        }

        ASTNode::AsyncFunctionStatement { name, body, .. } => {
            match get_identifier_string(&name.value) {
                Ok(func_name) => {
                    if let Some(&func_index) = context.function_map.get(&func_name) {
                        let offset = bytecode.current_offset();
                        bytecode.update_function_offset(func_index, offset);

                        // Create new variable scope for function
                        let old_var_map = context.variable_map.clone();
                        let old_var_counter = context.variable_counter;

                        for stmt in body {
                            generate_instructions(bytecode, context, stmt);
                        }

                        if let Some(return_opcode) = bytecode.get_opcode("return") {
                            bytecode.emit_instruction(return_opcode);
                        }

                        // Restore previous scopes
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
            generate_instructions(bytecode, context, expression);
        }

        ASTNode::IfExpression {
            condition,
            then_branch,
            else_branch,
        } => {
            generate_instructions(bytecode, context, condition);

            if let Some(jump_if_false_opcode) = bytecode.get_opcode("jump_if_false") {
                let false_jump_addr = bytecode.current_offset();
                bytecode.emit_instruction_u16(jump_if_false_opcode, 0);

                for stmt in then_branch {
                    generate_instructions(bytecode, context, stmt);
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
                        generate_instructions(bytecode, context, stmt);
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

        ASTNode::LetStatement { name, initializer } => {
            generate_let_statement(bytecode, context, name, initializer);
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
            (ConstantValue::Null, ConstantValue::Null) => return i as u16,
            _ => continue,
        }
    }

    bytecode.add_constant(value)
}

fn patch_jump_offset(bytecode: &mut BytecodeProgram, jump_addr: u32, target_addr: u32) {
    let offset = target_addr - jump_addr - 3;
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
            // Generate initializer expression and store in variable
            generate_instructions(bytecode, context, initializer);
            store_variable(bytecode, context, &variable_name);
        }
        Err(err) => {
            eprintln!("Compile error: {}", err);
        }
    }
}

/// Store a value from stack into a variable
fn store_variable(bytecode: &mut BytecodeProgram, context: &mut CompileContext, var_name: &str) {
    // Get or create variable index
    let var_index = if let Some(&index) = context.variable_map.get(var_name) {
        index
    } else {
        let index = context.variable_counter;
        context.variable_map.insert(var_name.to_string(), index);
        context.variable_counter += 1;
        index
    };

    // Emit store_var instruction
    if let Some(store_opcode) = bytecode.get_opcode("store_var") {
        bytecode.emit_instruction_u16(store_opcode, var_index);
    }
}

/// Load a variable value onto the stack
fn load_variable(bytecode: &mut BytecodeProgram, context: &CompileContext, var_name: &str) {
    if let Some(&var_index) = context.variable_map.get(var_name) {
        if let Some(load_opcode) = bytecode.get_opcode("load_var") {
            bytecode.emit_instruction_u16(load_opcode, var_index);
        }
    } else {
        eprintln!("Compile error: Variable '{}' not found", var_name);
    }
}

/// Generate bytecode for enum constructor
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

    // Look up the enum in the context
    if let Some(&enum_index) = context.enum_map.get(&enum_name_str) {
        // Generate bytecode for all field values first (push them onto stack)
        for value in values {
            generate_instructions(bytecode, context, value);
        }

        // Find the variant index within the enum
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
                // Validate field count matches
                if values.len() != field_names.len() {
                    eprintln!(
                        "Compile error: Enum constructor field count mismatch. Expected {}, got {}",
                        field_names.len(),
                        values.len()
                    );
                    return;
                }

                // Emit create_enum instruction with:
                // - enum_index: which enum type
                // - variant_index: which variant within the enum
                // - field_count: how many fields to pop from stack
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

fn compile(ast: ASTProgram) -> BytecodeProgram {
    compile_program(ast)
}

/// Check if the import path represents a local file
fn is_local_file_import(module_path: &str) -> bool {
    // Check for common local file patterns
    module_path.contains("/") ||           // Has path separators
    module_path.contains("\\") ||          // Windows path separators  
    module_path.starts_with("./") ||       // Relative path
    module_path.starts_with("../") ||      // Parent directory
    module_path.ends_with(".mir") ||       // Explicit file extension
    Path::new(module_path).exists() // File exists in filesystem
}

/// Load a module from a local file
fn load_local_module(module_path: &str, bytecode: &mut BytecodeProgram) -> Option<LoadedModule> {
    // Resolve the actual file path
    let file_path = resolve_module_path(module_path)?;

    // Read and parse the file
    let source = fs::read_to_string(&file_path).ok()?;
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program();

    if parser.had_error {
        eprintln!("Parse errors in module: {}", file_path);
        return None;
    }

    // Extract module definition from AST
    let module_def = extract_module_definition(&ast, module_path);

    // Create loaded module using the same logic as registry modules
    let mut loaded_module = LoadedModule {
        definition: module_def.clone(),
        function_indices: HashMap::new(),
        constant_indices: HashMap::new(),
    };

    // Add module constants to bytecode
    for (i, constant) in module_def.constants.iter().enumerate() {
        let const_name = format!("{}._const_{}", module_path, i);
        let const_index = bytecode.add_constant(constant.clone());
        loaded_module
            .constant_indices
            .insert(const_name, const_index);
    }

    // Add module functions to bytecode
    for func in &module_def.functions {
        let func_index = bytecode.add_function(func.arg_count, 0, 0); // Local count will be determined later
        loaded_module
            .function_indices
            .insert(func.name.clone(), func_index);

        // Add function name as constant for runtime lookup
        let func_name_const = format!("{}.{}", module_path, func.name);
        bytecode.add_constant(ConstantValue::String(func_name_const));
    }

    Some(loaded_module)
}

/// Resolve module path to actual file path
fn resolve_module_path(module_path: &str) -> Option<String> {
    // If it already has .mir extension, use as-is
    if module_path.ends_with(".mir") {
        if Path::new(module_path).exists() {
            return Some(module_path.to_string());
        }
    }

    // Try adding .mir extension
    let with_extension = format!("{}.mir", module_path);
    if Path::new(&with_extension).exists() {
        return Some(with_extension);
    }

    // Try relative paths
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

/// Extract module definition from AST (exported functions and constants)
fn extract_module_definition(ast: &ASTProgram, module_name: &str) -> ModuleDefinition {
    let mut functions = Vec::new();
    let constants = Vec::new();

    // Extract all function statements as exported functions
    for node in &ast.nodes {
        match node {
            ASTNode::FunctionStatement { name, params, .. } => {
                match get_identifier_string(&name.value) {
                    Ok(func_name) => {
                        functions.push(ModuleFunction {
                            name: func_name,
                            arg_count: params.len() as u8,
                            is_native: false, // User-defined function
                            native_id: None,
                        });
                    }
                    Err(err) => {
                        eprintln!("Compile error: {}", err);
                    }
                }
            }
            ASTNode::AsyncFunctionStatement { name, params, .. } => {
                match get_identifier_string(&name.value) {
                    Ok(func_name) => {
                        functions.push(ModuleFunction {
                            name: func_name,
                            arg_count: params.len() as u8,
                            is_native: false, // User-defined async function
                            native_id: None,
                        });
                    }
                    Err(err) => {
                        eprintln!("Compile error: {}", err);
                    }
                }
            }
            // Could extend to extract constants, enums, etc.
            _ => {}
        }
    }

    ModuleDefinition {
        name: module_name.to_string(),
        functions,
        constants,
    }
}
