use std::collections::HashMap;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    StoreVar(usize, usize) = 0x01,
    LoadVar(usize, usize) = 0x02,
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
    Not = 0x17,
    CreateArray(usize) = 0x18, // Create array with N elements from stack
    Jump(usize) = 0x20,
    JumpIfFalse(usize) = 0x21,
    JumpIfTrue(usize) = 0x22,
    Pop = 0x30,
    Push(Value) = 0x31,
    Dup = 0x32,
    Halt = 0x33,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarOutput {
    Created { index: usize, depth: usize },
    GotCurrentScope { index: usize, depth: usize },
    GotOuterScope { index: usize, depth: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Function { params: Vec<String>, offset: usize },
    HeapPointer(usize),
}

impl Value {
    pub fn type_name_stack(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Function { .. } => "function",
            Value::HeapPointer(_) => "heap pointer",
        }
    }

    pub fn type_name<'a>(&'a self, heap: &'a [HeapObject]) -> &'static str {
        match self {
            Value::HeapPointer(idx) => match heap.get(*idx) {
                Some(HeapObject::String(_)) => "string",
                Some(HeapObject::Number(_)) => "number",
                Some(HeapObject::Boolean(_)) => "boolean",
                Some(HeapObject::Null) => "null",
                Some(HeapObject::Array(_)) => "array",
                Some(HeapObject::Object(_)) => "object",
                None => "unknown",
            },
            _ => self.type_name_stack(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeapObject {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<HeapObject>),
    Object(HashMap<String, HeapObject>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ByteCode {
    pub constants: Vec<Value>,
    pub functions: Vec<Value>,
    pub instructions: Vec<Instruction>,
    pub instruction_lines: Vec<usize>,
}
