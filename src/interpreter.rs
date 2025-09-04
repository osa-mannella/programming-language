use crate::compiler::Compiler;
use crate::types::compiler::{ByteCode, HeapObject, Instruction, Value};
use crate::types::constants::{
    GC_CHECK_INTERVAL, GC_HISTORY_BUFFER_SIZE, GC_THRESHOLD, HEAP_SCORE_ARRAY_BASE,
    HEAP_SCORE_ARRAY_PER_ELEMENT, HEAP_SCORE_MAP_BASE, HEAP_SCORE_MAP_PER_ELEMENT,
    HEAP_SCORE_OTHER_OBJECT, HEAP_SCORE_STRING_BASE, MAX_STRING_LENGTH, UNDERFLOW_ERROR,
};
use crate::types::traits::IntoResult;
use std::collections::VecDeque;

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
    instruction_lines: Vec<usize>,
    heap: Vec<HeapObject>,
    last_heap_score: VecDeque<usize>,
    raw_compiler: Compiler,
}

impl VirtualMachine {
    pub fn new(bytecode: ByteCode, compiler: Compiler) -> Self {
        let vm = Self {
            stack: Vec::new(),
            stack_frames: vec![StackFrame::new()],
            return_addresses: Vec::new(),
            pc: 0,
            raw_compiler: compiler,
            constants: bytecode.constants,
            functions: bytecode.functions,
            instructions: bytecode.instructions,
            instruction_lines: bytecode.instruction_lines,
            heap: Vec::new(),
            last_heap_score: VecDeque::new(),
        };
        vm
    }

    fn gc(&mut self) {
        // Mark phase: Find all live objects by tracing from stack variables
        let mut marked = vec![false; self.heap.len()];
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
                    heap_score += HEAP_SCORE_ARRAY_BASE + arr.len() * HEAP_SCORE_ARRAY_PER_ELEMENT;
                }
                HeapObject::String(s) => {
                    heap_score += HEAP_SCORE_STRING_BASE + s.len();
                }
                HeapObject::Object(map) => {
                    heap_score += HEAP_SCORE_MAP_BASE + map.len() * HEAP_SCORE_MAP_PER_ELEMENT;
                }
                _ => {
                    heap_score += HEAP_SCORE_OTHER_OBJECT;
                }
            }
        }
        self.last_heap_score.push_back(heap_score);
        if self.last_heap_score.len() > GC_HISTORY_BUFFER_SIZE {
            self.last_heap_score.pop_front();
        }
        heap_score
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.pc < self.instructions.len() {
            if (self.pc + 1) % GC_CHECK_INTERVAL == 0 {
                let heap_score = self.heap_score();
                if heap_score >= GC_THRESHOLD {
                    self.gc();
                }
            }
            match &self.instructions[self.pc] {
                Instruction::Halt => break,
                _ => {
                    if let Err(e) = self.execute_instruction() {
                        let line = self.instruction_lines.get(self.pc).cloned().unwrap_or(0);
                        return Err(format!("[line {}] {}", line, e));
                    }
                }
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
                let value = self.stack.pop().ok_or(UNDERFLOW_ERROR)?;

                self.set_variable(*var_index, value)?;
            }

            Instruction::LoadVar(depth, var_index) => {
                let value = self.resolve_variable(*depth, *var_index)?;
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
                let b = self.stack.pop().ok_or(UNDERFLOW_ERROR)?;
                let a = self.stack.pop().ok_or(UNDERFLOW_ERROR)?;

                match (&a, &b) {
                    (Value::Number(a_num), Value::Number(b_num)) => {
                        self.stack.push(Value::Number(a_num + b_num));
                    }
                    (Value::String(a_str), Value::String(b_str)) => {
                        let result = format!("{}{}", a_str, b_str);
                        self.stack.push(Value::String(result));
                    }
                    _ => {
                        return Err(format!(
                            "Cannot add {} and {} - both operands must be the same type",
                            a.type_name(&self.heap),
                            b.type_name(&self.heap)
                        ));
                    }
                }
            }

            Instruction::Sub => {
                let b: f64 = self.pop_value()?;
                let a: f64 = self.pop_value()?;
                self.stack.push(Value::Number(a - b));
            }

            Instruction::Mul => {
                let b: f64 = self.pop_value()?;
                let a: f64 = self.pop_value()?;
                self.stack.push(Value::Number(a * b));
            }

            Instruction::Div => {
                let b: f64 = self.pop_value()?;
                let a: f64 = self.pop_value()?;
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                self.stack.push(Value::Number(a / b));
            }

            Instruction::Equal => {
                const STACK_UNDERFLOW: &str = UNDERFLOW_ERROR;
                let b: Value = self.stack.pop().ok_or(STACK_UNDERFLOW)?;
                let a: Value = self.stack.pop().ok_or(STACK_UNDERFLOW)?;
                let result = self.values_equal(&a, &b);
                self.stack
                    .push(Value::Boolean(if result { true } else { false }));
            }

            Instruction::Less => {
                let b: f64 = self.pop_value()?;
                let a: f64 = self.pop_value()?;
                self.stack
                    .push(Value::Boolean(if a < b { true } else { false }));
            }

            Instruction::Greater => {
                let b: f64 = self.pop_value()?;
                let a: f64 = self.pop_value()?;
                self.stack
                    .push(Value::Boolean(if a > b { true } else { false }));
            }

            Instruction::Not => {
                let value = self.stack.pop().ok_or(UNDERFLOW_ERROR)?;
                match value {
                    Value::Boolean(b) => {
                        self.stack.push(Value::Boolean(!b));
                    }
                    _ => {
                        return Err(format!(
                            "Logical NOT operation requires boolean operand, got {}",
                            value.type_name_stack()
                        ));
                    }
                }
            }

            Instruction::CreateArray(size) => {
                let mut elements = Vec::new();
                for _ in 0..*size {
                    let element = self.stack.pop().ok_or(UNDERFLOW_ERROR)?;
                    elements.push(self.value_to_heap_object(element));
                }
                elements.reverse();

                let heap_array = HeapObject::Array(elements);
                self.heap.push(heap_array);
                let heap_index = self.heap.len() - 1;
                self.stack.push(Value::HeapPointer(heap_index));
            }

            Instruction::Jump(addr) => {
                self.pc = *addr;
                return Ok(());
            }

            Instruction::JumpIfFalse(addr) => {
                let value: bool = self.pop_value()?;
                if value == false {
                    self.pc = *addr;
                    return Ok(());
                }
            }

            Instruction::JumpIfTrue(addr) => {
                let value: bool = self.pop_value()?;
                if value == true {
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
                self.stack.pop().ok_or(UNDERFLOW_ERROR)?;
            }

            Instruction::Dup => {
                let value = self.stack.last().ok_or(UNDERFLOW_ERROR)?.clone();
                self.stack.push(value);
            }

            Instruction::Halt => {
                return Ok(());
            }
        }

        self.pc += 1;
        Ok(())
    }

    fn resolve_variable(&self, depth: usize, var_index: usize) -> Result<Value, String> {
        for frame in self.stack_frames.iter().rev() {
            if let Some(value) = frame.get_variable(var_index) {
                return Ok(value.clone());
            }
        }
        if let Some(scope) = self.raw_compiler.variables.get(depth) {
            for (name, idx) in scope.iter() {
                if *idx == var_index {
                    return Err(format!(
                        "Variable '{}' (index {}) not found",
                        name, var_index
                    ));
                }
            }
        }
        Err(format!("Variable with index {} not found", var_index))
    }

    fn heap_push(&mut self, value: Value) -> Option<Value> {
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

    fn pop_value<T>(&mut self) -> Result<T, String>
    where
        Value: IntoResult<T>,
    {
        match self.stack.pop() {
            Some(value) => value.into_result(),
            None => Err(UNDERFLOW_ERROR.to_string()),
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

    fn value_to_heap_object(&self, value: Value) -> HeapObject {
        match value {
            Value::Number(n) => HeapObject::Number(n),
            Value::String(s) => HeapObject::String(s),
            Value::Boolean(b) => HeapObject::Boolean(b),
            Value::HeapPointer(_) => HeapObject::Null, // Could preserve references, but simplify for now
            Value::Function { .. } => HeapObject::Null, // Functions can't go in arrays yet
        }
    }
}
