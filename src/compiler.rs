use crate::types::ast::*;
use std::collections::HashMap;
use std::fmt;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    StoreVar(usize) = 0x01,
    LoadVar(usize) = 0x02,
    LoadArg(usize) = 0x03,
    Call(usize) = 0x04,
    Return = 0x05,
    LoadConst(usize) = 0x06,
    Add = 0x10,
    Sub = 0x11,
    Div = 0x12,
    Mul = 0x13,
    Equal = 0x14,
    Less = 0x15,
    Greater = 0x16,
    Jump(usize) = 0x20,
    JumpIfFalse(usize) = 0x21,
    JumpIfTrue(usize) = 0x22,
    Pop = 0x30,
    Dup = 0x31,
    Halt = 0x32,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Function { params: Vec<String>, offset: usize },
}

pub struct Compiler {
    constants: Vec<Value>,
    functions: HashMap<String, usize>,
    function_table: Vec<Value>,
    variables: HashMap<String, usize>,
    instructions: Vec<Instruction>,
    current_function: Option<String>,
    current_function_params: HashMap<String, usize>, // Function-local parameter mapping
    current_param_count: usize, // Track number of parameters in current function
}

pub struct ByteCode {
    pub constants: Vec<Value>,
    pub functions: Vec<Value>,
    pub instructions: Vec<Instruction>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            functions: HashMap::new(),
            function_table: Vec::new(),
            variables: HashMap::new(),
            instructions: Vec::new(),
            current_function: None,
            current_function_params: HashMap::new(),
            current_param_count: 0,
        }
    }

    pub fn compile(&mut self, program: &Program) -> ByteCode {
        self.collect_pass(&program.statements);
        self.generate_instructions(&program.statements);
        self.instructions.push(Instruction::Halt);

        ByteCode {
            constants: self.constants.clone(),
            functions: self.function_table.clone(),
            instructions: self.instructions.clone(),
        }
    }

    fn collect_pass(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            match stmt {
                Stmt::Func { name, params, body } => {
                    let function_index = self.function_table.len();
                    self.functions.insert(name.clone(), function_index);

                    let function_value = Value::Function {
                        params: params.clone(),
                        offset: 0,
                    };
                    self.function_table.push(function_value);
                    self.collect_pass(body);
                }
                Stmt::Let { value, .. } => {
                    self.collect_constants_from_expr(value);
                }
                Stmt::Expr(expr) => {
                    self.collect_constants_from_expr(expr);
                }
            }
        }
    }

    fn collect_constants_from_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                let value = Value::Number(*n);
                if !self
                    .constants
                    .iter()
                    .any(|c| matches!((c, &value), (Value::Number(a), Value::Number(b)) if a == b))
                {
                    self.constants.push(value);
                }
            }
            Expr::String(s) => {
                let value = Value::String(s.clone());
                if !self
                    .constants
                    .iter()
                    .any(|c| matches!((c, &value), (Value::String(a), Value::String(b)) if a == b))
                {
                    self.constants.push(value);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_constants_from_expr(left);
                self.collect_constants_from_expr(right);
            }
            Expr::Call { func, args } => {
                self.collect_constants_from_expr(func);
                for arg in args {
                    self.collect_constants_from_expr(arg);
                }
            }
            Expr::Pipeline { left, right } => {
                self.collect_constants_from_expr(left);
                self.collect_constants_from_expr(right);
            }
            Expr::Identifier(_) => {}
        }
    }

    fn generate_instructions(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            self.compile_statement(stmt, false);
        }
    }

    fn compile_statement(&mut self, stmt: &Stmt, last: bool) {
        match stmt {
            Stmt::Let { name, value } => {
                self.compile_expression(value);
                let var_index = self.get_or_create_variable_index(name);
                self.instructions.push(Instruction::StoreVar(var_index));
            }
            Stmt::Func { name, params, body } => {
                // Jump over the function definition in main execution flow
                let jump_over_function = self.instructions.len();
                self.instructions.push(Instruction::Jump(0)); // Placeholder, will be patched

                if let Some(function_index) = self.functions.get(name).cloned() {
                    if let Some(Value::Function { params, .. }) =
                        self.function_table.get_mut(function_index)
                    {
                        let param_count = params.len();
                        let params = params.clone();
                        self.function_table[function_index] = Value::Function {
                            params,
                            offset: self.instructions.len(), // Function starts here
                        };

                        // Generate LOAD_ARG instruction at function entry
                        if param_count > 0 {
                            self.instructions.push(Instruction::LoadArg(param_count));
                        }
                    }
                }

                let old_function = self.current_function.clone();
                let old_params = self.current_function_params.clone();
                let old_variables = self.variables.clone();
                let old_param_count = self.current_param_count;

                self.current_function = Some(name.clone());
                self.current_param_count = params.len();

                // Set up function parameter mapping with local indices
                self.current_function_params.clear();
                self.variables.clear(); // Clear variables for function scope

                for (param_index, param_name) in params.iter().enumerate() {
                    self.current_function_params
                        .insert(param_name.clone(), param_index);
                }

                for (i, body_stmt) in body.iter().enumerate() {
                    let last = i == body.len() - 1;
                    self.compile_statement(body_stmt, last);
                }

                self.instructions.push(Instruction::Return);
                self.current_function = old_function;
                self.current_function_params = old_params;
                self.variables = old_variables;
                self.current_param_count = old_param_count;

                // Patch the jump to skip over the function
                let after_function = self.instructions.len();
                self.instructions[jump_over_function] = Instruction::Jump(after_function);
            }
            Stmt::Expr(expr) => {
                self.compile_expression(expr);
                if !last {
                    self.instructions.push(Instruction::Pop);
                }
            }
        }
    }

    fn compile_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                let const_index = self.get_constant_index(&Value::Number(*n));
                self.instructions.push(Instruction::LoadConst(const_index));
            }
            Expr::String(s) => {
                let const_index = self.get_constant_index(&Value::String(s.clone()));
                self.instructions.push(Instruction::LoadConst(const_index));
            }
            Expr::Identifier(name) => {
                let var_index = self.get_or_create_variable_index(name);
                self.instructions.push(Instruction::LoadVar(var_index));
            }
            Expr::Binary { left, op, right } => {
                self.compile_expression(left);
                self.compile_expression(right);
                match op {
                    BinaryOp::Add => self.instructions.push(Instruction::Add),
                    BinaryOp::Sub => self.instructions.push(Instruction::Sub),
                    BinaryOp::Mul => self.instructions.push(Instruction::Mul),
                    BinaryOp::Div => self.instructions.push(Instruction::Div),
                    BinaryOp::Eq => self.instructions.push(Instruction::Equal),
                    BinaryOp::Lt => self.instructions.push(Instruction::Less),
                    BinaryOp::Gt => self.instructions.push(Instruction::Greater),
                    BinaryOp::Ne => {
                        self.instructions.push(Instruction::Equal);
                    }
                    BinaryOp::Le => {
                        self.instructions.push(Instruction::Greater);
                    }
                    BinaryOp::Ge => {
                        self.instructions.push(Instruction::Less);
                    }
                }
            }
            Expr::Call { func, args } => {
                for arg in args.iter().rev() {
                    self.compile_expression(arg);
                }

                if let Expr::Identifier(func_name) = func.as_ref() {
                    if let Some(function_index) = self.functions.get(func_name).cloned() {
                        self.instructions.push(Instruction::Call(function_index));
                    }
                } else {
                    self.compile_expression(func);
                }
            }
            Expr::Pipeline { left, right } => {
                self.compile_expression(left);

                match right.as_ref() {
                    Expr::Call { func, args } => {
                        for arg in args.iter().rev() {
                            self.compile_expression(arg);
                        }

                        if let Expr::Identifier(func_name) = func.as_ref() {
                            if let Some(function_index) = self.functions.get(func_name).cloned() {
                                self.instructions.push(Instruction::Call(function_index));
                            }
                        }
                    }
                    _ => {
                        self.compile_expression(right);
                    }
                }
            }
        }
    }

    fn get_constant_index(&self, value: &Value) -> usize {
        self.constants
            .iter()
            .position(|c| match (c, value) {
                (Value::Number(a), Value::Number(b)) => a == b,
                (Value::String(a), Value::String(b)) => a == b,
                _ => false,
            })
            .unwrap_or(0)
    }

    fn get_or_create_variable_index(&mut self, name: &str) -> usize {
        // First check if this is a function parameter in the current function
        if self.current_function.is_some() {
            if let Some(param_index) = self.current_function_params.get(name) {
                return *param_index;
            }
        }

        // For local variables in functions, start indexing after parameters
        if self.current_function.is_some() {
            // If it exists we just return the de-referenced index
            if let Some(index) = self.variables.get(name) {
                *index
            } else {
                // Local variables start from param_count to avoid conflicts
                // Here we create the variable
                let index = self.current_param_count + self.variables.len();
                self.variables.insert(name.to_string(), index);
                index
            }
        } else {
            // Global scope - use standard indexing
            if let Some(index) = self.variables.get(name) {
                *index
            } else {
                let index = self.variables.len();
                self.variables.insert(name.to_string(), index);
                index
            }
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::StoreVar(idx) => write!(f, "STORE_VAR {}", idx),
            Instruction::LoadVar(idx) => write!(f, "LOAD_VAR {}", idx),
            Instruction::LoadArg(idx) => write!(f, "LOAD_ARG {}", idx),
            Instruction::Call(idx) => write!(f, "CALL {}", idx),
            Instruction::Return => write!(f, "RETURN"),
            Instruction::LoadConst(idx) => write!(f, "LOAD_CONST {}", idx),
            Instruction::Add => write!(f, "ADD"),
            Instruction::Sub => write!(f, "SUB"),
            Instruction::Div => write!(f, "DIV"),
            Instruction::Mul => write!(f, "MUL"),
            Instruction::Equal => write!(f, "EQUAL"),
            Instruction::Less => write!(f, "LESS"),
            Instruction::Greater => write!(f, "GREATER"),
            Instruction::Jump(addr) => write!(f, "JUMP {}", addr),
            Instruction::JumpIfFalse(addr) => write!(f, "JUMP_IF_FALSE {}", addr),
            Instruction::JumpIfTrue(addr) => write!(f, "JUMP_IF_TRUE {}", addr),
            Instruction::Pop => write!(f, "POP"),
            Instruction::Dup => write!(f, "DUP"),
            Instruction::Halt => write!(f, "HALT"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Function { params, offset } => {
                write!(f, "fn({}) @{}", params.join(", "), offset)
            }
        }
    }
}

impl fmt::Display for ByteCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== BYTECODE ===")?;

        writeln!(f, "\nConstants:")?;
        for (i, constant) in self.constants.iter().enumerate() {
            writeln!(f, "  [{}] {}", i, constant)?;
        }

        writeln!(f, "\nFunctions:")?;
        for (i, function) in self.functions.iter().enumerate() {
            writeln!(f, "  [{}] {}", i, function)?;
        }

        writeln!(f, "\nInstructions:")?;
        for (i, instruction) in self.instructions.iter().enumerate() {
            writeln!(f, "  {:04}: {}", i, instruction)?;
        }

        Ok(())
    }
}
