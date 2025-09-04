# n Language Test Suite

This directory contains test files for the n programming language, integrated with Rust's built-in testing framework.

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Specific Test

```bash
cargo test test_basic_arithmetic
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Tests in Release Mode

```bash
cargo test --release
```

## Test Files

- **`basic_arithmetic.n`** - Basic arithmetic operations
- **`comparison_operators.n`** - Comparison operators
- **`string_operations.n`** - String operations and heap allocation
- **`function_definitions.n`** - Function definitions and calls
- **`complex_expressions.n`** - Complex expression evaluation
- **`heap_stress.n`** - Heap allocation and garbage collection
- **`edge_cases.n`** - Edge cases and boundary conditions
- **`nested_functions.n`** - Nested function definitions
- **`array_operations.n`** - Array creation and manipulation
- **`error_cases.n`** - Error conditions (should fail)

## Test Categories

### ✅ Currently Passing

- Basic arithmetic operations
- Comparison operators
- Complex expressions
- Heap stress testing
- Nested functions
- Array creation and operations
- Error detection

### 🔧 Currently Failing (Expected)

- String operations (string concatenation not implemented)
- Function definitions with strings (string operations issue)
- Edge cases (parser issue with empty function parameters)

## Integration with Rust Testing

The tests are implemented as Rust unit tests in `src/tests.rs` and automatically:

- Build the n compiler
- Execute test files
- Verify expected outcomes
- Report results in standard Rust test format
- Integrate with CI/CD pipelines
- Support test filtering and parallel execution

## Adding New Tests

1. Create a new `.n` file in the `tests/` directory
2. Add a corresponding test function in `src/tests.rs`
3. Run `cargo test` to verify

This approach provides better integration with Rust tooling and follows standard practices for Rust projects.
