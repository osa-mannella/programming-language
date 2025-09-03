use crate::compiler::{ByteCode, HeapObject, Instruction, Value};
use std::collections::VecDeque;

const GC_CHECK_INTERVAL: usize = 12;

#[derive(Debug, Clone)]
pub struct StackFrame {
    variables: Vec<Value>,
}

impl StackFrame {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

    pub fn set_variable(&mut self, index: usize, value: Value) {
        while index >= self.variables.len() {
            self.variables.push(Value::Number(0.0));
        }
        self.variables[index] = value;
    }

    pub fn get_variable(&self, index: usize) -> Option<&Value> {
        self.variables.get(index)
    }
}

pub struct VirtualMachine {
    stack: Vec<Value>,
    stack_frames: Vec<StackFrame>,
    return_addresses: Vec<usize>,
    pc: usize,
    constants: Vec<Value>,
    functions: Vec<Value>,
    instructions: Vec<Instruction>,
    heap: Vec<HeapObject>,
    last_heap_score: VecDeque<usize>,
}

impl VirtualMachine {
    pub fn new(bytecode: ByteCode) -> Self {
        let vm = Self {
            stack: Vec::new(),
            stack_frames: vec![StackFrame::new()],
            return_addresses: Vec::new(),
            pc: 0,
            constants: bytecode.constants,
            functions: bytecode.functions,
            instructions: bytecode.instructions,
            heap: Vec::new(),
            last_heap_score: VecDeque::new(),
        };
        vm
    }

    fn gc(&mut self) {
        // Mark phase: Find all live objects by tracing from stack variables
        let mut marked = vec![false; self.heap.len()];
        println!("Mark phase: {}", marked.len());
        for frame in &self.stack_frames {
            for value in &frame.variables {
                if let Value::HeapPointer(idx) = value {
                    if *idx < marked.len() {
                        marked[*idx] = true;
                    }
                }
            }
        }

        // Sweep phase: Build new compacted heap and create index mapping
        let mut new_heap = Vec::with_capacity(self.heap.len());
        let mut remap = vec![None; self.heap.len()];
        for (i, (obj, is_marked)) in self.heap.iter().zip(marked.iter()).enumerate() {
            if *is_marked {
                remap[i] = Some(new_heap.len());
                new_heap.push(obj.clone());
            }
        }

        // Update phase: Fix all heap pointer references to use new indices
        for frame in &mut self.stack_frames {
            for value in &mut frame.variables {
                if let Value::HeapPointer(idx) = value {
                    if *idx < remap.len() {
                        if let Some(new_idx) = remap[*idx] {
                            *value = Value::HeapPointer(new_idx);
                        }
                    }
                }
            }
        }

        // Replace old heap with compacted heap
        self.heap = new_heap;
    }

    fn heap_score(&mut self) -> usize {
        let mut heap_score: usize = 0;
        for obj in &self.heap {
            match obj {
                HeapObject::Array(arr) => {
                    heap_score += 16 + arr.len() * 8;
                }
                HeapObject::String(s) => {
                    heap_score += 24 + s.len();
                }
                HeapObject::Object(map) => {
                    heap_score += 32 + map.len() * 16;
                }
                _ => {
                    heap_score += 32;
                }
            }
        }
        self.last_heap_score.push_back(heap_score);
        if self.last_heap_score.len() > 10 {
            self.last_heap_score.pop_front();
        }
        heap_score
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.pc < self.instructions.len() {
            println!("PC: {}, GC_CHECK_INTERVAL: {}", self.pc, GC_CHECK_INTERVAL);
            if (self.pc + 1) % GC_CHECK_INTERVAL == 0 {
                let heap_score = self.heap_score();
                if heap_score > 1000 {
                    self.gc();
                }
            }
            match &self.instructions[self.pc] {
                Instruction::Halt => break,
                _ => self.execute_instruction()?,
            }
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), String> {
        match &self.instructions[self.pc].clone() {
            Instruction::Push(value) => {
                self.stack.push(value.clone());
            }

            Instruction::LoadConst(index) => {
                let value = self
                    .constants
                    .get(*index)
                    .ok_or("Invalid constant index")?
                    .clone();
                self.stack.push(value);
            }

            Instruction::StoreVar(_, var_index) => {
                let value = self.stack.pop().ok_or("Stack underflow")?;

                self.set_variable(*var_index, value)?;
            }

            Instruction::LoadVar(_, var_index) => {
                let value = self.resolve_variable(*var_index)?;
                self.stack.push(value);
            }

            Instruction::LoadArg(arg_count) => {
                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(self.stack.pop().ok_or("Not enough arguments")?);
                }
                for (param_index, arg_value) in args.iter().rev().enumerate() {
                    self.set_variable(param_index, arg_value.clone())?;
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
                    self.return_addresses.push(self.pc + 1);

                    let new_frame = StackFrame::new();
                    self.stack_frames.push(new_frame);

                    self.pc = *offset;
                    return Ok(());
                } else {
                    return Err("Invalid function value".to_string());
                }
            }

            Instruction::Return => {
                if self.stack_frames.len() > 1 {
                    self.stack_frames.pop();
                }

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
        for frame in self.stack_frames.iter().rev() {
            if let Some(value) = frame.get_variable(var_index) {
                return Ok(value.clone());
            }
        }

        Err(format!("Variable with index {} not found", var_index))
    }

    fn heap_push(&mut self, value: Value) -> Option<Value> {
        const MAX_STRING_LENGTH: usize = 1024;
        let heap_index = match &value {
            Value::String(s) if s.len() > MAX_STRING_LENGTH => {
                let heap_obj = HeapObject::String(s.clone());
                self.heap.push(heap_obj);
                Some(self.heap.len() - 1)
            }
            _ => None,
        };

        heap_index.map(|index| Value::HeapPointer(index))
    }

    fn set_variable(&mut self, var_index: usize, value: Value) -> Result<(), String> {
        let final_value = match self.heap_push(value.clone()) {
            Some(heap_pointer) => heap_pointer,
            None => value,
        };

        let current_frame = self
            .stack_frames
            .last_mut()
            .ok_or("No stack frame available")?;

        current_frame.set_variable(var_index, final_value);
        Ok(())
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
        println!("Heap: {:?}", self.heap);

        if let Some(current_instruction) = self.instructions.get(self.pc) {
            println!("Next Instruction: {:?}", current_instruction);
        }
        println!("================");
    }
}
