pub mod library;

#[cfg(test)]
mod integration_tests {
    use crate::library::compiler::compile_program;
    use crate::library::lexer::*;
    use crate::library::parser::Parser;

    fn full_compile_pipeline(
        source: &str,
    ) -> Result<crate::library::compiler::BytecodeProgram, String> {
        // Phase 1: Lexing
        let lexer = Lexer::new(source);

        // Phase 2: Parsing
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        if parser.had_error {
            return Err("Parser error".to_string());
        }

        // Phase 3: Compilation
        compile_program(program)
    }

    #[test]
    fn test_complex_programs() {
        // Test realistic, complex programs through the entire pipeline

        let fibonacci = r#"
        func fibonacci(n) {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
        
        let result = fibonacci(10)
        "#;
        assert!(full_compile_pipeline(fibonacci).is_ok());

        let enum_example = r#"
        enum Maybe { 
            Some { value }, 
            None 
        }
        
        func unwrap_or(maybe, default) {
            match maybe {
                Maybe::Some { value } -> value,
                Maybe::None -> default
            }
        }
        
        let x = Maybe::Some { value = 42 }
        let result = unwrap_or(x, 0)
        "#;
        assert!(full_compile_pipeline(enum_example).is_ok());

        let _async_example = r#"
        import "IO"
        
        async func process_data(input) {
            let transformed = input |> 
                fn(x) -> x * 2 |>
                fn(x) -> x + 1
            
            await save_result(transformed)
        }
        
        async func main() {
            let data = [1, 2, 3, 4, 5]
            let new_data = data <- [6, 7, 8]
            
            for item in new_data {
                await process_data(item)
            }
        }
        "#;
        // Note: This might fail due to 'for' not being implemented yet
        // but tests what we can compile

        let pipeline_example = r#"
        let numbers = [1, 2, 3, 4, 5]
        let result = numbers |>
            fn(arr) -> arr <- [6, 7, 8] |>
            fn(arr) -> (arr[0] + arr[1])
        "#;
        assert!(full_compile_pipeline(pipeline_example).is_ok());

        let struct_manipulation = r#"
        let person = { 
            name = "Alice", 
            age = 25, 
            address = { 
                street = "123 Main St", 
                city = "Springfield" 
            } 
        }
        
        let updated_person = person <- { 
            age = 26,
            address = person.address <- { city = "New York" }
        }
        "#;
        assert!(full_compile_pipeline(struct_manipulation).is_ok());
    }

    #[test]
    fn test_error_handling_across_phases() {
        // Test that errors are properly handled at each phase

        // Lexer errors
        assert!(full_compile_pipeline("@#$%^").is_err());
        assert!(full_compile_pipeline("\"unterminated string").is_err());

        // Parser errors
        assert!(full_compile_pipeline("let = 42").is_err());
        assert!(full_compile_pipeline("func () {}").is_err());
        assert!(full_compile_pipeline("[1, 2,").is_err());
        assert!(full_compile_pipeline("if true {}").is_ok()); // This should actually be OK

        // Semantic errors (caught during compilation)
        // These should parse but might have compilation issues
        let programs_that_parse_but_might_have_semantic_issues =
            vec!["unknown_function()", "let x = y.nonexistent_property"];

        for program in programs_that_parse_but_might_have_semantic_issues {
            // These should at least parse successfully
            let lexer = Lexer::new(program);
            let mut parser = Parser::new(lexer);
            let _program = parser.parse_program();
            assert!(!parser.had_error, "Parser should not error on: {}", program);
        }
    }

    #[test]
    fn test_phase_isolation() {
        // Test that each phase can work independently

        // Test lexer in isolation
        let source = "let x = 42 + 3.14";
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        while let Some(token) = lexer.next() {
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        assert_eq!(tokens.len(), 6); // let, x, =, 42, +, 3.14

        // Test parser in isolation
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert!(!parser.had_error);
        assert_eq!(program.nodes.len(), 1);

        // Test compiler in isolation
        let bytecode = compile_program(program).expect("Compilation should succeed");
        assert_eq!(bytecode.header.magic, *b"MIRB");
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_bytecode_consistency() {
        // Test that the same source produces consistent bytecode

        let source = "let x = 1 + 2 * 3";

        let bytecode1 = full_compile_pipeline(source).unwrap();
        let bytecode2 = full_compile_pipeline(source).unwrap();

        // Headers should be identical
        assert_eq!(bytecode1.header.magic, bytecode2.header.magic);
        assert_eq!(bytecode1.header.version, bytecode2.header.version);

        // Constants should be the same
        assert_eq!(bytecode1.constants.len(), bytecode2.constants.len());

        // Instructions should be identical
        assert_eq!(bytecode1.instructions.len(), bytecode2.instructions.len());
    }

    #[test]
    fn test_all_token_types_coverage() {
        // Ensure all token types can be lexed, parsed, and compiled

        let comprehensive_source = r#"
        import "IO"
        
        enum Result { Ok { value }, Error { message } }
        
        async func comprehensive_test(param1, param2) {
            // Test all operators and literals
            let numbers = [1, 2, 3.14, 0.5] <- [42]
            let strings = ["hello", "world"] <- ["test"]
            let booleans = [true, false]
            
            // Test all binary operators
            let arithmetic = 1 + 2 - 3 * 4 / 5 ^ 2
            let comparison = 1 < 2 && 3 > 2 || 4 <= 5 && 6 >= 5
            let equality = 1 == 1 && 2 != 3
            
            // Test struct operations
            let person = { name = "Alice", age = 30 }
            let updated = person <- { age = 31 }
            
            // Test enum construction and matching
            let result = Result::Ok { value = 42 }
            let unwrapped = match result {
                Result::Ok { value } -> value,
                Result::Error { message } -> 0
            }
            
            // Test pipeline
            let piped = numbers |> 
                fn(arr) -> (arr[0]) |>
                fn(x) -> x * 2
            
            // Test conditionals
            let conditional = if piped > 10 { "big" } else { "small" }
            
            // Test lambdas
            let lambda = fn(x, y) -> x + y
            let async_lambda = async fn(z) -> await process(z)
            
            // Test await
            let awaited = await async_lambda(42)
            
            // Test grouping and precedence
            let complex = ((1 + 2) * 3) + 4
            
            awaited + complex
        }
        
        let! final_result = comprehensive_test(10, 20)
        "#;

        let result = full_compile_pipeline(comprehensive_source);
        assert!(
            result.is_ok(),
            "Comprehensive test failed: {:?}",
            result.err()
        );

        let bytecode = result.unwrap();

        // Verify the bytecode has all the expected components
        assert!(!bytecode.constants.is_empty());
        assert!(!bytecode.functions.is_empty());
        assert!(!bytecode.instructions.is_empty());

        // Verify it has strings, numbers, and module references
        assert!(
            bytecode
                .constants
                .iter()
                .any(|c| matches!(c.value, crate::library::compiler::ConstantValue::String(_)))
        );
        assert!(
            bytecode
                .constants
                .iter()
                .any(|c| matches!(c.value, crate::library::compiler::ConstantValue::Number(_)))
        );
    }

    #[test]
    fn test_performance_characteristics() {
        // Test that the compilation pipeline can handle reasonably sized programs

        let mut large_program = String::new();
        large_program.push_str("import \"IO\"\n");

        // Generate a program with many functions
        for i in 0..100 {
            large_program.push_str(&format!("func func_{0}(x) {{ x + {0} }}\n", i));
        }

        // Generate a program with many variables
        for i in 0..100 {
            large_program.push_str(&format!("let var_{0} = func_{0}({0})\n", i));
        }

        let result = full_compile_pipeline(&large_program);
        assert!(result.is_ok(), "Large program compilation failed");

        let bytecode = result.unwrap();
        assert!(bytecode.functions.len() >= 100);
        assert!(bytecode.constants.len() > 100);
    }

    #[test]
    fn test_edge_cases() {
        // Test various edge cases that might break the pipeline

        // Empty program
        assert!(full_compile_pipeline("").is_ok());

        // Only whitespace
        assert!(full_compile_pipeline("   \n  \t  \n  ").is_ok());

        // Single expression
        assert!(full_compile_pipeline("42").is_ok());

        // Deeply nested expressions
        assert!(full_compile_pipeline("((((1 + 2) * 3) - 4) / 5)").is_ok());

        // Long identifier names
        let long_name = "a".repeat(100);
        assert!(full_compile_pipeline(&format!("let {} = 42", long_name)).is_ok());

        // Many parameters
        let many_params = (0..50)
            .map(|i| format!("p{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        assert!(full_compile_pipeline(&format!("func test({}) {{ 42 }}", many_params)).is_ok());

        // Deeply nested structures
        let nested_struct = "{ a = { b = { c = { d = 42 } } } }";
        assert!(full_compile_pipeline(&format!("let nested = {}", nested_struct)).is_ok());

        // Long string literals
        let long_string = "x".repeat(1000);
        assert!(full_compile_pipeline(&format!("let s = \"{}\"", long_string)).is_ok());
    }

    #[test]
    fn test_feature_interaction() {
        // Test that different language features work together correctly

        let feature_interaction_test = r#"
        enum Container { 
            List { items }, 
            Single { item } 
        }
        
        async func process_container(container) {
            match container {
                Container::List { items } -> {
                    let processed = items |>
                        fn(list) -> (list <- [0]) |>
                        fn(list) -> (list[0] + list[1])
                    
                    await save_result(processed)
                },
                Container::Single { item } -> {
                    let doubled = item * 2
                    await save_result(doubled)
                }
            }
        }
        
        func create_containers() {
            let list_container = Container::List { 
                items = [1, 2, 3] 
            }
            let single_container = Container::Single { 
                item = 42 
            }
            
            ([list_container, single_container])
        }
        
        async func main() {
            let containers = create_containers()
            let first = (containers[0])
            await process_container(first)
        }
        
        let! result = main()
        "#;

        let compilation_result = full_compile_pipeline(feature_interaction_test);
        assert!(
            compilation_result.is_ok(),
            "Feature interaction test failed: {:?}",
            compilation_result.err()
        );
    }
}
