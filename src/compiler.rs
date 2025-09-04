use crate::types::ast::*;
use std::collections::HashMap;
use std::fmt;

use crate::types::compiler::*;

pub struct Compiler {
    pub constants: Vec<Value>,
    pub functions: HashMap<String, usize>,
    pub function_table: Vec<Value>,
    pub variables: Vec<HashMap<String, usize>>,
    pub instructions: Vec<Instruction>,
    pub instruction_lines: Vec<usize>,
    pub current_function: Option<String>,
    pub depth: usize,
    pub in_new_function: bool,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            functions: HashMap::new(),
            function_table: Vec::new(),
            variables: Vec::new(),
            depth: 0,
            instructions: Vec::new(),
            instruction_lines: Vec::new(),
            current_function: None,
            in_new_function: false,
        }
    }

    fn insert_variable(&mut self, name: &str) -> usize {
        while self.variables.len() <= self.depth {
            self.variables.push(HashMap::new());
        }

        if self.in_new_function {
            self.variables[self.depth].clear();
            self.in_new_function = false;
        }

        let current_scope = &mut self.variables[self.depth];
        let local_index = current_scope.len(); // Next available index in this scope
        current_scope.insert(name.to_string(), local_index);

        local_index
    }

    fn get_variable(&self, name: &str) -> Option<(usize, usize)> {
        let mut result = None;
        for (depth, scope) in self.variables.iter().enumerate() {
            if depth > self.depth {
                break;
            }
            if let Some(index) = scope.get(name) {
                result = Some((*index, depth));
            }
        }
        result
    }

    pub fn compile(&mut self, program: &Program) -> Result<ByteCode, String> {
        self.collect_pass(&program.statements);
        self.generate_instructions(&program.statements)?;
        self.instructions.push(Instruction::Halt);
        self.instruction_lines.push(self.current_line());

        Ok(ByteCode {
            constants: self.constants.clone(),
            functions: self.function_table.clone(),
            instructions: self.instructions.clone(),
            instruction_lines: self.instruction_lines.clone(),
        })
    }

    fn collect_pass(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            match stmt {
                Stmt::Func {
                    name, params, body, ..
                } => {
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
                Stmt::Expr(expr, _) => {
                    self.collect_constants_from_expr(expr);
                }
            }
        }
    }

    fn collect_constants_from_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Boolean(b) => {
                let value = Value::Boolean(*b);
                if !self.constants.iter().any(
                    |c| matches!((c, &value), (Value::Boolean(a), Value::Boolean(b)) if a == b),
                ) {
                    self.constants.push(value);
                }
            }
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
            Expr::Unary { right, .. } => {
                self.collect_constants_from_expr(right);
            }
            Expr::Array { elements } => {
                for element in elements {
                    self.collect_constants_from_expr(element);
                }
            }
            Expr::Identifier(_) => {}
        }
    }

    fn generate_instructions(&mut self, statements: &[Stmt]) -> Result<(), String> {
        for stmt in statements {
            self.compile_statement(stmt, false)?;
        }
        Ok(())
    }

    fn compile_statement(&mut self, stmt: &Stmt, last: bool) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value, line } => {
                self.compile_expression(value)?;
                let var_index = match self.get_or_create_variable_index(name) {
                    VarOutput::Created { index, .. } => index,
                    VarOutput::GotCurrentScope { .. } => {
                        return Err(format!(
                            "Variable '{}' is already defined in the current scope",
                            name
                        ));
                    }
                    VarOutput::GotOuterScope { .. } => self.insert_variable(name),
                };

                self.push_with_line(Instruction::StoreVar(self.depth, var_index), *line);
                if last {
                    self.push_with_line(Instruction::Push(Value::Number(0.0)), *line); // TEMP MEASURE, REPLACE THIS ONCE ENUMS ARE IMPLEMENTED PLEASE !!!
                }
            }
            Stmt::Func {
                name,
                params,
                body,
                line,
            } => {
                let jump_over_function = self.instructions.len();
                self.push_with_line(Instruction::Jump(0), *line);
                self.depth += 1;
                self.in_new_function = true;
                if let Some(function_index) = self.functions.get(name).cloned() {
                    if let Some(Value::Function { params, .. }) =
                        self.function_table.get_mut(function_index)
                    {
                        let param_count = params.len();
                        let params = params.clone();
                        self.function_table[function_index] = Value::Function {
                            params,
                            offset: self.instructions.len(),
                        };

                        if param_count > 0 {
                            self.push_with_line(Instruction::LoadArg(param_count), *line);
                        }
                    }
                }

                let old_function = self.current_function.clone();

                self.current_function = Some(name.clone());

                for param_name in params.iter() {
                    let _ = self.get_or_create_variable_index(param_name);
                }

                for (i, body_stmt) in body.iter().enumerate() {
                    let last = i == body.len() - 1;
                    self.compile_statement(body_stmt, last)?;
                }
                self.depth -= 1;

                self.push_with_line(Instruction::Return, *line);
                self.current_function = old_function;

                let after_function = self.instructions.len();
                self.instructions[jump_over_function] = Instruction::Jump(after_function);
            }
            Stmt::Expr(expr, line) => {
                self.compile_expression(expr)?;
                if !last {
                    self.push_with_line(Instruction::Pop, *line);
                }
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Boolean(b) => {
                let const_index = self.get_constant_index(&Value::Boolean(*b));
                self.push(Instruction::LoadConst(const_index));
            }
            Expr::Number(n) => {
                let const_index = self.get_constant_index(&Value::Number(*n));
                self.push(Instruction::LoadConst(const_index));
            }
            Expr::String(s) => {
                let const_index = self.get_constant_index(&Value::String(s.clone()));
                self.push(Instruction::LoadConst(const_index));
            }
            Expr::Identifier(name) => {
                let (var_index, fetch_depth) = match self.get_or_create_variable_index(name) {
                    VarOutput::Created { index, depth } => (index, depth),
                    VarOutput::GotCurrentScope { index, depth } => (index, depth),
                    VarOutput::GotOuterScope { index, depth } => (index, depth),
                };
                self.push(Instruction::LoadVar(fetch_depth, var_index));
            }
            Expr::Binary { left, op, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                match op {
                    BinaryOp::Add => self.push(Instruction::Add),
                    BinaryOp::Sub => self.push(Instruction::Sub),
                    BinaryOp::Mul => self.push(Instruction::Mul),
                    BinaryOp::Div => self.push(Instruction::Div),
                    BinaryOp::Eq => self.push(Instruction::Equal),
                    BinaryOp::Lt => self.push(Instruction::Less),
                    BinaryOp::Gt => self.push(Instruction::Greater),
                    BinaryOp::Ne => {
                        self.push(Instruction::Equal);
                    }
                    BinaryOp::Le => {
                        self.push(Instruction::Greater);
                    }
                    BinaryOp::Ge => {
                        self.push(Instruction::Less);
                    }
                }
            }
            Expr::Call { func, args } => {
                for arg in args.iter().rev() {
                    self.compile_expression(arg)?;
                }

                if let Expr::Identifier(func_name) = func.as_ref() {
                    if let Some(function_index) = self.functions.get(func_name).cloned() {
                        self.push(Instruction::Call(function_index));
                    } else {
                        // Defer to runtime: try to call offset 0, which will fail at runtime
                        // or simply ignore here and compile callee expression for dynamic resolution when implemented
                    }
                } else {
                    self.compile_expression(func)?;
                }
            }
            Expr::Pipeline { left, right } => {
                self.compile_expression(left)?;

                match right.as_ref() {
                    Expr::Call { func, args } => {
                        for arg in args.iter().rev() {
                            self.compile_expression(arg)?;
                        }
                        if let Expr::Identifier(func_name) = func.as_ref() {
                            if let Some(function_index) = self.functions.get(func_name).cloned() {
                                self.push(Instruction::Call(function_index));
                            }
                        }
                    }
                    Expr::Identifier(func_name) => {
                        if let Some(function_index) = self.functions.get(func_name).cloned() {
                            self.push(Instruction::Call(function_index));
                        }
                    }
                    _ => {
                        println!("right: {:?}", right);
                        self.compile_expression(right)?;
                    }
                }
            }
            Expr::Unary { op, right } => match op {
                UnaryOp::Neg => {
                    self.push(Instruction::Push(Value::Number(0.0)));
                    self.compile_expression(right)?;
                    self.push(Instruction::Sub);
                }
                UnaryOp::Not => {
                    self.compile_expression(right)?;
                    self.push(Instruction::Not);
                }
            },
            Expr::Array { elements } => {
                for element in elements.iter() {
                    self.compile_expression(element)?;
                }
                self.push(Instruction::CreateArray(elements.len()));
            }
        }
        Ok(())
    }

    fn get_constant_index(&self, value: &Value) -> usize {
        self.constants
            .iter()
            .position(|c| match (c, value) {
                (Value::Number(a), Value::Number(b)) => a == b,
                (Value::String(a), Value::String(b)) => a == b,
                (Value::Boolean(a), Value::Boolean(b)) => a == b,
                _ => false,
            })
            .unwrap_or(0)
    }

    fn get_or_create_variable_index(&mut self, name: &str) -> VarOutput {
        if let Some((index, depth)) = self.get_variable(name) {
            if depth == self.depth {
                VarOutput::GotCurrentScope { index, depth }
            } else {
                VarOutput::GotOuterScope { index, depth }
            }
        } else {
            let index = self.insert_variable(name);
            VarOutput::Created {
                index,
                depth: self.depth,
            }
        }
    }
}

impl Compiler {
    fn current_line(&self) -> usize {
        *self.instruction_lines.last().unwrap_or(&1)
    }

    fn push(&mut self, instr: Instruction) {
        // In expression contexts, we don't have a precise line; reuse last known line or 1.
        let line = self.current_line();
        self.instructions.push(instr);
        self.instruction_lines.push(line);
    }

    fn push_with_line(&mut self, instr: Instruction, line: usize) {
        self.instructions.push(instr);
        self.instruction_lines.push(line);
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Push(value) => write!(f, "PUSH {}", value),
            Instruction::StoreVar(scope, idx) => write!(f, "STORE_VAR {} {}", scope, idx),
            Instruction::LoadVar(scope, idx) => write!(f, "LOAD_VAR {} {}", scope, idx),
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
            Instruction::Not => write!(f, "NOT"),
            Instruction::CreateArray(size) => write!(f, "CREATE_ARRAY {}", size),
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
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Function { params, offset } => {
                write!(f, "fn({}) @{}", params.join(", "), offset)
            }
            Value::HeapPointer(idx) => write!(f, "HEAP_POINTER {}", idx),
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
