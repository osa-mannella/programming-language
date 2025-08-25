use crate::compiler::{ByteCode, Instruction, Value};

#[derive(Debug, Clone)]
pub struct StackFrame {
    variables: Vec<usize>, // Array of variable indexes for O(1) access
}

impl StackFrame {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

    pub fn set_variable(&mut self, index: usize, value_index: usize) {
        if index >= self.variables.len() {
            self.variables.resize(index + 1, 0);
        }
        self.variables[index] = value_index;
    }

    pub fn get_variable(&self, index: usize) -> Option<usize> {
        self.variables.get(index).copied()
    }
}

pub struct VirtualMachine {
    stack: Vec<Value>,
    stack_frames: Vec<StackFrame>, // 2D array system: [global_frame, local_frames...]
    return_addresses: Vec<usize>,
    pc: usize, // Program counter
    constants: Vec<Value>,
    functions: Vec<Value>,
    instructions: Vec<Instruction>,
    global_variables: Vec<Value>, // Runtime variable storage
}

impl VirtualMachine {
    pub fn new(bytecode: ByteCode) -> Self {
        let vm = Self {
            stack: Vec::new(),
            stack_frames: vec![StackFrame::new()], // Start with global frame
            return_addresses: Vec::new(),
            pc: 0,
            constants: bytecode.constants,
            functions: bytecode.functions,
            instructions: bytecode.instructions,
            global_variables: Vec::new(),
        };
        vm
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.pc < self.instructions.len() {
            match &self.instructions[self.pc] {
                Instruction::Halt => break,
                _ => self.execute_instruction()?,
            }
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), String> {
        match &self.instructions[self.pc].clone() {
            Instruction::LoadConst(index) => {
                let value = self
                    .constants
                    .get(*index)
                    .ok_or("Invalid constant index")?
                    .clone();
                self.stack.push(value);
            }

            Instruction::StoreVar(var_index) => {
                let value = self.stack.pop().ok_or("Stack underflow")?;

                // Store in global variables and get the storage index
                let storage_index = self.global_variables.len();
                self.global_variables.push(value);

                // Store the storage index in current stack frame
                let current_frame = self
                    .stack_frames
                    .last_mut()
                    .ok_or("No stack frame available")?;
                current_frame.set_variable(*var_index, storage_index);
            }

            Instruction::LoadVar(var_index) => {
                let value = self.resolve_variable(*var_index)?;
                self.stack.push(value);
            }

            Instruction::LoadArg(arg_count) => {
                // Pop arguments from stack (they were pushed in reverse order)
                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(
                        self.stack
                            .pop()
                            .ok_or("Not enough arguments for LOAD_ARG instruction")?,
                    );
                }

                // Get current stack frame (should be the function's frame)
                let current_frame = self
                    .stack_frames
                    .last_mut()
                    .ok_or("No stack frame available for LOAD_ARG")?;

                // Bind arguments to local variables 0..arg_count-1 (reverse args to maintain order)
                for (param_index, arg_value) in args.iter().rev().enumerate() {
                    let storage_index = self.global_variables.len();
                    self.global_variables.push(arg_value.clone());
                    current_frame.set_variable(param_index, storage_index);
                }
            }

            Instruction::Add => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack.push(Value::Number(a + b));
            }

            Instruction::Sub => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack.push(Value::Number(a - b));
            }

            Instruction::Mul => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack.push(Value::Number(a * b));
            }

            Instruction::Div => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                self.stack.push(Value::Number(a / b));
            }

            Instruction::Equal => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                let result = self.values_equal(&a, &b);
                self.stack
                    .push(Value::Number(if result { 1.0 } else { 0.0 }));
            }

            Instruction::Less => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack
                    .push(Value::Number(if a < b { 1.0 } else { 0.0 }));
            }

            Instruction::Greater => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack
                    .push(Value::Number(if a > b { 1.0 } else { 0.0 }));
            }

            Instruction::Jump(addr) => {
                self.pc = *addr;
                return Ok(());
            }

            Instruction::JumpIfFalse(addr) => {
                let value = self.pop_number()?;
                if value == 0.0 {
                    self.pc = *addr;
                    return Ok(());
                }
            }

            Instruction::JumpIfTrue(addr) => {
                let value = self.pop_number()?;
                if value != 0.0 {
                    self.pc = *addr;
                    return Ok(());
                }
            }

            Instruction::Call(func_index) => {
                let function = self
                    .functions
                    .get(*func_index)
                    .ok_or("Invalid function index")?;

                if let Value::Function { offset, .. } = function {
                    // Push return address
                    self.return_addresses.push(self.pc + 1);

                    // Create new stack frame for function
                    let new_frame = StackFrame::new();
                    self.stack_frames.push(new_frame);

                    // Jump to function (LOAD_ARG will handle parameter binding)
                    self.pc = *offset;
                    return Ok(());
                } else {
                    return Err("Invalid function value".to_string());
                }
            }

            Instruction::Return => {
                // Pop current stack frame
                if self.stack_frames.len() > 1 {
                    self.stack_frames.pop();
                }

                // Jump back to return address
                if let Some(return_addr) = self.return_addresses.pop() {
                    self.pc = return_addr;
                    return Ok(());
                } else {
                    return Err("No return address available".to_string());
                }
            }

            Instruction::Pop => {
                self.stack.pop().ok_or("Stack underflow")?;
            }

            Instruction::Dup => {
                let value = self.stack.last().ok_or("Stack underflow")?.clone();
                self.stack.push(value);
            }

            Instruction::Halt => {
                return Ok(());
            }
        }

        self.pc += 1;
        Ok(())
    }

    fn resolve_variable(&self, var_index: usize) -> Result<Value, String> {
        // Check local scope first (last stack frame)
        if let Some(local_frame) = self.stack_frames.last() {
            if let Some(storage_index) = local_frame.get_variable(var_index) {
                return Ok(self
                    .global_variables
                    .get(storage_index)
                    .ok_or("Invalid variable storage index")?
                    .clone());
            }
        }

        // Check global scope (first stack frame)
        if let Some(global_frame) = self.stack_frames.first() {
            if let Some(storage_index) = global_frame.get_variable(var_index) {
                return Ok(self
                    .global_variables
                    .get(storage_index)
                    .ok_or("Invalid variable storage index")?
                    .clone());
            }
        }

        Err(format!("Variable with index {} not found", var_index))
    }

    fn pop_number(&mut self) -> Result<f64, String> {
        match self.stack.pop() {
            Some(Value::Number(n)) => Ok(n),
            Some(_) => Err("Expected number on stack".to_string()),
            None => Err("Stack underflow".to_string()),
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            _ => false,
        }
    }

    pub fn debug_stack(&self) {
        println!("=== VM DEBUG ===");
        println!("PC: {}", self.pc);
        println!("Stack: {:?}", self.stack);
        println!("Stack Frames: {}", self.stack_frames.len());
        println!("Global Variables: {:?}", self.global_variables);

        if let Some(current_instruction) = self.instructions.get(self.pc) {
            println!("Next Instruction: {:?}", current_instruction);
        }
        println!("================");
    }
}
