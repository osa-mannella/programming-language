use crate::types::compiler::Value;

pub trait IntoResult<T> {
    fn into_result(self) -> Result<T, String>;
}

impl IntoResult<f64> for Value {
    fn into_result(self) -> Result<f64, String> {
        match self {
            Value::Number(n) => Ok(n),
            _ => Err("Expected number on stack".to_string()),
        }
    }
}

impl IntoResult<String> for Value {
    fn into_result(self) -> Result<String, String> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err("Expected string on stack".to_string()),
        }
    }
}

impl IntoResult<bool> for Value {
    fn into_result(self) -> Result<bool, String> {
        match self {
            Value::Boolean(b) => Ok(b),
            _ => Err("Expected boolean on stack".to_string()),
        }
    }
}
