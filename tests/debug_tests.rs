use mirrow::library::debug::print_bytecode_debug;
use mirrow::library::compiler::compile_program;
use mirrow::library::lexer::Lexer;
use mirrow::library::parser::Parser;
use mirrow::library::ast::ASTProgram;

#[test]
fn test_struct_create_debug_output() {
    let source = r#"{ name = "Alice", age = 30 }"#;
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    
    if let Some(node) = parser.parse_expression(0) {
        if !parser.had_error {
            let program = ASTProgram { nodes: vec![node] };
            let bytecode = compile_program(program).unwrap();
            
            // Test that debug output works without crashing
            print_bytecode_debug(&bytecode);
            
            // Verify that struct_create opcode is present
            assert!(bytecode.get_opcode("struct_create").is_some());
        }
    }
}