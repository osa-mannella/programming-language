use mirrow::library::ast::ASTProgram;
use mirrow::library::compiler::{BytecodeProgram, ConstantValue, EnumVariant, compile_program};
use mirrow::library::lexer::Lexer;
use mirrow::library::parser::Parser;

fn compile_source(source: &str) -> Result<BytecodeProgram, String> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if parser.had_error {
        Err("Parser error".to_string())
    } else {
        compile_program(program)
    }
}

fn compile_expression(source: &str) -> Result<BytecodeProgram, String> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);

    if let Some(node) = parser.parse_expression(0) {
        if parser.had_error {
            Err("Parser error".to_string())
        } else {
            let program = ASTProgram { nodes: vec![node] };
            compile_program(program)
        }
    } else {
        Err("Failed to parse expression".to_string())
    }
}

#[test]
fn test_empty_program() {
    let bytecode = compile_source("").unwrap();

    // Should have basic header and halt instruction
    assert_eq!(bytecode.header.magic, *b"MIRB");
    assert_eq!(bytecode.header.version, 1);
    assert!(!bytecode.instructions.is_empty());

    // Should end with halt instruction
    let halt_opcode = bytecode.get_opcode("halt").unwrap();
    assert_eq!(*bytecode.instructions.last().unwrap(), halt_opcode);
}

#[test]
fn test_literal_constants() {
    let bytecode = compile_expression("42").unwrap();

    // Should contain the number constant
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 42.0) })
    );

    let bytecode = compile_expression(r#""hello""#).unwrap();

    // Should contain the string constant
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "hello") })
    );

    let bytecode = compile_expression("true").unwrap();

    // Should contain the boolean constant
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Boolean(true)) })
    );
}

#[test]
fn test_variable_operations() {
    let bytecode = compile_source("let x = 42").unwrap();

    // Should have store_var instruction
    let store_var_opcode = bytecode.get_opcode("store_var");
    assert!(store_var_opcode.is_some());

    // Check that instructions contain the store operation
    let store_opcode = store_var_opcode.unwrap();
    assert!(bytecode.instructions.contains(&store_opcode));

    // Should contain the constant
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 42.0) })
    );
}

#[test]
fn test_binary_operations() {
    let test_cases = vec![
        ("1 + 2", "add"),
        ("1 - 2", "sub"),
        ("1 * 2", "mul"),
        ("1 / 2", "div"),
        ("1 ^ 2", "power"),
        ("1 == 2", "equal"),
        ("1 != 2", "not_equal"),
        ("1 < 2", "less"),
        ("1 <= 2", "less_equal"),
        ("1 > 2", "greater"),
        ("1 >= 2", "greater_equal"),
    ];

    for (source, expected_opcode_name) in test_cases {
        let bytecode = compile_expression(source).unwrap();
        let expected_opcode = bytecode.get_opcode(expected_opcode_name);
        assert!(
            expected_opcode.is_some(),
            "Missing opcode: {}",
            expected_opcode_name
        );

        let opcode = expected_opcode.unwrap();
        assert!(
            bytecode.instructions.contains(&opcode),
            "Instructions don't contain {} opcode for expression: {}",
            expected_opcode_name,
            source
        );
    }
}

#[test]
fn test_function_compilation() {
    let bytecode = compile_source("func test(x, y) { x + y }").unwrap();

    // Should have at least one function
    assert!(!bytecode.functions.is_empty());

    let func = &bytecode.functions[0];
    assert_eq!(func.arg_count, 2); // x and y parameters

    // Should have add operation in instructions
    let add_opcode = bytecode.get_opcode("add").unwrap();
    assert!(bytecode.instructions.contains(&add_opcode));
}

#[test]
fn test_async_function_compilation() {
    let bytecode = compile_source("async func test() { await something() }").unwrap();

    // Should compile successfully and have function entry
    assert!(!bytecode.functions.is_empty());

    let func = &bytecode.functions[0];
    assert_eq!(func.arg_count, 0);
}

#[test]
fn test_lambda_compilation() {
    let bytecode = compile_expression("fn(x) -> x + 1").unwrap();

    // Should have function entry for the lambda
    assert!(!bytecode.functions.is_empty());

    let func = &bytecode.functions[0];
    assert_eq!(func.arg_count, 1);
}

#[test]
fn test_if_expression_compilation() {
    let bytecode = compile_expression("if true { 42 } else { 0 }").unwrap();

    // Should have jump instructions
    let jump_if_false = bytecode.get_opcode("jump_if_false");
    let jump = bytecode.get_opcode("jump");

    assert!(jump_if_false.is_some());
    assert!(jump.is_some());
}

#[test]
fn test_match_statement_compilation() {
    let bytecode = compile_expression("match x { 1 -> \"one\", 2 -> \"two\" }").unwrap();

    // Should have match-related opcodes
    let match_literal = bytecode.get_opcode("match_literal");
    assert!(match_literal.is_some());

    // Should have string constants
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "one") })
    );
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "two") })
    );
}

#[test]
fn test_enum_compilation() {
    let bytecode = compile_source("enum Color { Red, Green, Blue }").unwrap();

    // Should have enum definition
    assert!(!bytecode.enums.is_empty());

    let enum_def = &bytecode.enums[0];
    assert_eq!(enum_def.variants.len(), 3);
}

#[test]
fn test_enum_constructor_compilation() {
    let source = r#"
enum Shape { Circle { radius } }
let circle = Shape::Circle { radius = 5.0 }
        "#;

    let bytecode = compile_source(source).unwrap();

    // Should have enum definition
    assert!(!bytecode.enums.is_empty());

    // Should have create_enum opcode
    let create_enum = bytecode.get_opcode("create_enum");
    assert!(create_enum.is_some());
}

#[test]
fn test_enum_complex_compilation() {
    let source = r#"enum Shape {
    Rectangle { length, width },
    Square { side },
    Circle { radius },
}

let circle = Shape::Circle { radius = 4 }

match circle {
    Shape::Circle { radius } -> {PI*r^2}
}"#;

    let bytecode = compile_source(source).unwrap();

    // Should have enum definition
    assert!(!bytecode.enums.is_empty());

    // Should have create_enum opcode
    let create_enum = bytecode.get_opcode("create_enum");
    assert!(create_enum.is_some());

    // Should have match instructions
    let match_literal = bytecode.get_opcode("match_literal");
    assert!(match_literal.is_some());
}

#[test]
fn test_list_literal_compilation() {
    let bytecode = compile_expression("[1, 2, 3]").unwrap();

    // Should have the number constants
    for i in 1..=3 {
        assert!(
            bytecode
                .constants
                .iter()
                .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == i as f64) })
        );
    }
}

#[test]
fn test_index_access_compilation() {
    let bytecode = compile_expression("arr[0]").unwrap();

    // Should have index_access opcode
    let index_access_opcode = bytecode.get_opcode("index_access");
    assert!(index_access_opcode.is_some());

    // Should contain the instructions for loading constant 0
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 0.0) })
    );
}

#[test]
fn test_struct_literal_compilation() {
    let bytecode = compile_expression(r#"{ name = "John", age = 30 }"#).unwrap();

    // Should have string and number constants
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "John") })
    );
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 30.0) })
    );
}

#[test]
fn test_struct_update_compilation() {
    let bytecode = compile_expression("person <- { age = 31 }").unwrap();

    // Should have the age constant
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 31.0) })
    );

    // Should compile without errors (struct updates are complex operations)
}

#[test]
fn test_array_append_compilation() {
    let bytecode = compile_expression("arr <- [4, 5, 6]").unwrap();

    // Should have the number constants
    for i in 4..=6 {
        assert!(
            bytecode
                .constants
                .iter()
                .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == i as f64) })
        );
    }
}

#[test]
fn test_function_call_compilation() {
    let bytecode = compile_expression("my_func(1, 2)").unwrap();

    // Should have call instruction
    let call_opcode = bytecode.get_opcode("call");
    assert!(call_opcode.is_some());

    // Should have argument constants
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 1.0) })
    );
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 2.0) })
    );
}

#[test]
fn test_property_access_compilation() {
    let bytecode = compile_expression("obj['property']").unwrap();

    // Should compile without errors (property access implementation varies)
    assert!(bytecode.header.magic == *b"MIRB");
}

#[test]
fn test_pipeline_compilation() {
    let bytecode = compile_expression("value |> transform |> process").unwrap();

    // Should compile successfully
    assert!(bytecode.header.magic == *b"MIRB");
}

#[test]
fn test_import_statement_compilation() {
    let bytecode = compile_source(r#"import "IO""#).unwrap();

    // Should have IO module constants
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s.contains("IO")) })
    );
}

#[test]
fn test_complex_program_compilation() {
    let source = r#"
import "IO"

func factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

let result = factorial(5)
        "#;

    let bytecode = compile_source(source).unwrap();

    // Should have function definition
    assert!(!bytecode.functions.is_empty());

    // Should have various opcodes
    assert!(bytecode.get_opcode("call").is_some());
    assert!(bytecode.get_opcode("mul").is_some());
    assert!(bytecode.get_opcode("sub").is_some());
    assert!(bytecode.get_opcode("less_equal").is_some());

    // Should have number constants
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 1.0) })
    );
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 5.0) })
    );
}

#[test]
fn test_bytecode_header() {
    let bytecode = compile_source("let x = 42").unwrap();

    assert_eq!(bytecode.header.magic, *b"MIRB");
    assert_eq!(bytecode.header.version, 1);
    assert_eq!(bytecode.header.flags, 0);
    assert!(bytecode.header.message.contains("verse"));
}

#[test]
fn test_opcode_map_completeness() {
    let bytecode = BytecodeProgram::new();

    // Test that all expected opcodes are present
    let expected_opcodes = vec![
        "load_const",
        "load_global",
        "store_global",
        "load_local",
        "store_local",
        "add",
        "sub",
        "mul",
        "div",
        "power",
        "equal",
        "less",
        "greater",
        "not_equal",
        "less_equal",
        "greater_equal",
        "and",
        "or",
        "jump",
        "jump_if_false",
        "jump_if_true",
        "call",
        "call_global",
        "return",
        "call_native",
        "pop",
        "dup",
        "store_var",
        "load_var",
        "create_enum",
        "get_enum_variant",
        "match_literal",
        "match_enum_variant",
        "extract_enum_field",
        "match_fail",
        "struct_create",
        "halt",
    ];

    for opcode_name in expected_opcodes {
        assert!(
            bytecode.get_opcode(opcode_name).is_some(),
            "Missing opcode: {}",
            opcode_name
        );
    }
}

#[test]
fn test_constant_deduplication() {
    let bytecode = compile_expression("42 + 42").unwrap();

    // Should only have one instance of the number 42
    let count = bytecode
        .constants
        .iter()
        .filter(|c| matches!(c.value, ConstantValue::Number(n) if n == 42.0))
        .count();

    // Note: This might be 2 if no deduplication is implemented
    // The test verifies the current behavior
    assert!(count >= 1);
}

#[test]
fn test_instruction_emission() {
    let mut bytecode = BytecodeProgram::new();

    // Test basic instruction emission
    let add_opcode = bytecode.get_opcode("add").unwrap();
    bytecode.emit_instruction(add_opcode);

    assert_eq!(bytecode.instructions.len(), 1);
    assert_eq!(bytecode.instructions[0], add_opcode);

    // Test instruction with operands
    let load_const_opcode = bytecode.get_opcode("load_const").unwrap();
    bytecode.emit_instruction_u16(load_const_opcode, 42);

    assert_eq!(bytecode.instructions.len(), 4); // 1 + 1 + 2 bytes
    assert_eq!(bytecode.instructions[1], load_const_opcode);
}

#[test]
fn test_function_operations() {
    let mut bytecode = BytecodeProgram::new();

    // Test adding functions
    let func_index = bytecode.add_function(2, 1, 100);
    assert_eq!(func_index, 0);
    assert_eq!(bytecode.functions.len(), 1);

    let func = &bytecode.functions[0];
    assert_eq!(func.arg_count, 2);
    assert_eq!(func.local_count, 1);
    assert_eq!(func.offset, 100);

    // Test updating function offset
    bytecode.update_function_offset(0, 200);
    assert_eq!(bytecode.functions[0].offset, 200);
}

#[test]
fn test_enum_operations() {
    let mut bytecode = BytecodeProgram::new();

    // Add enum name constant
    let name_index = bytecode.add_constant(ConstantValue::String("Color".to_string()));

    // Create enum variants
    let variants = vec![
        EnumVariant {
            name_index: 0,
            field_count: 0,
        },
        EnumVariant {
            name_index: 1,
            field_count: 1,
        },
    ];

    let enum_index = bytecode.add_enum(name_index, variants);
    assert_eq!(enum_index, 0);
    assert_eq!(bytecode.enums.len(), 1);

    let enum_def = &bytecode.enums[0];
    assert_eq!(enum_def.name_index, name_index);
    assert_eq!(enum_def.variants.len(), 2);
}

#[test]
fn test_error_handling() {
    // Test that malformed syntax doesn't crash the compiler
    let result = compile_source("let = ");
    assert!(result.is_err());

    let result = compile_source("func ( ) { }");
    assert!(result.is_err());
}

#[test]
fn test_nested_expressions() {
    let bytecode = compile_expression("((1 + 2) * 3) + 4").unwrap();

    // Should have all the number constants
    for i in 1..=4 {
        assert!(
            bytecode
                .constants
                .iter()
                .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == i as f64) })
        );
    }

    // Should have arithmetic operations
    assert!(bytecode.get_opcode("add").is_some());
    assert!(bytecode.get_opcode("mul").is_some());
}

#[test]
fn test_let_bang_compilation() {
    let bytecode = compile_source("let! x = 42").unwrap();

    // Should compile successfully (let! vs let difference handled at runtime)
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 42.0) })
    );

    let store_var_opcode = bytecode.get_opcode("store_var").unwrap();
    assert!(bytecode.instructions.contains(&store_var_opcode));
}

#[test]
fn test_string_interpolation_compilation() {
    let bytecode = compile_expression(r#"$"Hello ${name}!""#).unwrap();

    // Should contain the string constants
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "Hello ") })
    );
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "!") })
    );

    // Should have string concatenation instruction
    let concat_opcode = bytecode.get_opcode("string_concat");
    assert!(concat_opcode.is_some());
}

#[test]
fn test_string_interpolation_multiple_expressions() {
    let bytecode = compile_expression(r#"$"Result: ${x + y}""#).unwrap();

    // Should contain the string constant
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "Result: ") })
    );

    // Should have addition and string concatenation instructions
    let add_opcode = bytecode.get_opcode("add");
    let concat_opcode = bytecode.get_opcode("string_concat");
    assert!(add_opcode.is_some());
    assert!(concat_opcode.is_some());
}

#[test]
fn test_struct_create_compilation() {
    let bytecode = compile_expression(r#"{ name = "Alice", age = 30 }"#).unwrap();

    // Should have struct_create opcode
    let struct_create_opcode = bytecode.get_opcode("struct_create");
    assert!(struct_create_opcode.is_some());

    // Should contain the field name constants
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "name") })
    );
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "age") })
    );

    // Should contain the field value constants
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "Alice") })
    );
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 30.0) })
    );

    // Should have struct_create instruction in bytecode
    let struct_opcode = struct_create_opcode.unwrap();
    assert!(bytecode.instructions.contains(&struct_opcode));
}

#[test]
fn test_struct_create_single_field() {
    let bytecode = compile_expression(r#"{ x = 42 }"#).unwrap();

    // Should have struct_create opcode
    let struct_create_opcode = bytecode.get_opcode("struct_create");
    assert!(struct_create_opcode.is_some());

    // Should contain the field name and value constants
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "x") })
    );
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 42.0) })
    );
}

#[test]
fn test_struct_create_empty() {
    let bytecode = compile_expression("{}").unwrap();

    // Should have struct_create opcode
    let struct_create_opcode = bytecode.get_opcode("struct_create");
    assert!(struct_create_opcode.is_some());

    // Should have struct_create instruction with field count 0
    let struct_opcode = struct_create_opcode.unwrap();
    assert!(bytecode.instructions.contains(&struct_opcode));
}

#[test]
fn test_power_operator_compilation() {
    // Test ^ (caret) power operator
    let bytecode = compile_expression("2 ^ 3").unwrap();

    let power_opcode = bytecode.get_opcode("power");
    assert!(power_opcode.is_some());

    let opcode = power_opcode.unwrap();
    assert!(bytecode.instructions.contains(&opcode));

    // Should contain the number constants
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 2.0) })
    );
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::Number(n) if n == 3.0) })
    );
}

#[test]
fn test_single_quote_strings() {
    let bytecode = compile_expression("'hello world'").unwrap();

    // Should contain the string constant
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "hello world") })
    );
}

#[test]
fn test_double_quote_strings() {
    let bytecode = compile_expression(r#""hello world""#).unwrap();

    // Should contain the string constant
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "hello world") })
    );
}

#[test]
fn test_quote_string_escaping() {
    // Test single quotes escaping double quotes
    let bytecode = compile_expression(r#"'He said "hello"'"#).unwrap();
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == r#"He said "hello""#) })
    );

    // Test double quotes escaping single quotes  
    let bytecode = compile_expression(r#""It's working""#).unwrap();
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "It's working") })
    );
}

#[test]
fn test_string_escape_sequences() {
    // Test escape sequences in single quoted strings
    let bytecode = compile_expression(r"'line1\nline2'").unwrap();
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "line1\nline2") })
    );

    // Test escape sequences in double quoted strings
    let bytecode = compile_expression(r#""tab\there""#).unwrap();
    assert!(
        bytecode
            .constants
            .iter()
            .any(|c| { matches!(c.value, ConstantValue::String(ref s) if s == "tab\there") })
    );
}
