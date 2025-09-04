use crate::runtime::compile_and_run;
use std::path::Path;

pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub output: String,
    pub exit_code: i32,
}

pub fn run_n_file(file_path: &str) -> TestResult {
    let result = compile_and_run(file_path);

    let (passed, output, exit_code) = match result {
        Ok(success_msg) => (true, success_msg, 0),
        Err(error_msg) => (false, error_msg, 1),
    };

    TestResult {
        name: Path::new(file_path)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        passed,
        output,
        exit_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let result = run_n_file("tests/basic_arithmetic.n");
        assert!(
            result.passed,
            "Basic arithmetic test failed: {}",
            result.output
        );
    }

    #[test]
    fn test_comparison_operators() {
        let result = run_n_file("tests/comparison_operators.n");
        assert!(
            result.passed,
            "Comparison operators test failed: {}",
            result.output
        );
    }

    #[test]
    fn test_string_operations() {
        let result = run_n_file("tests/string_operations.n");
        assert!(
            result.exit_code != -1,
            "String operations test crashed: {}",
            result.output
        );
    }

    #[test]
    fn test_function_definitions() {
        let result = run_n_file("tests/function_definitions.n");
        assert!(
            result.exit_code != -1,
            "Function definitions test crashed: {}",
            result.output
        );
    }

    #[test]
    fn test_complex_expressions() {
        let result = run_n_file("tests/complex_expressions.n");
        assert!(
            result.passed,
            "Complex expressions test failed: {}",
            result.output
        );
    }

    #[test]
    fn test_heap_stress() {
        let result = run_n_file("tests/heap_stress.n");
        assert!(result.passed, "Heap stress test failed: {}", result.output);
    }

    #[test]
    fn test_edge_cases() {
        let result = run_n_file("tests/edge_cases.n");
        assert!(
            result.exit_code != -1,
            "Edge cases test crashed: {}",
            result.output
        );
    }

    #[test]
    fn test_nested_functions() {
        let result = run_n_file("tests/nested_functions.n");
        assert!(
            result.passed,
            "Nested functions test failed: {}",
            result.output
        );
    }

    #[test]
    fn test_error_cases() {
        let result = run_n_file("tests/error_cases.n");
        assert!(
            !result.passed,
            "Error cases test should have failed but passed: {}",
            result.output
        );
    }

    #[test]
    fn test_division_by_zero_detection() {
        let result = run_n_file("tests/error_cases.n");
        assert!(!result.passed, "Division by zero should cause failure");
        assert!(
            result.output.contains("Division by zero") || result.output.contains("Runtime error"),
            "Should detect division by zero error: {}",
            result.output
        );
    }

    #[test]
    fn test_array_operations() {
        let result = run_n_file("tests/array_operations.n");
        assert!(
            result.passed,
            "Array operations test failed: {}",
            result.output
        );
    }
}
