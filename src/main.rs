mod compiler;
mod debug;
mod interpreter;
mod lexer;
mod parser;
mod types;

#[cfg(test)]
mod tests;

pub mod runtime {
    use crate::compiler::Compiler;
    use crate::interpreter::VirtualMachine;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    pub fn compile_and_run(filename: &str) -> Result<String, String> {
        compile_and_run_with_debug(filename, false)
    }

    pub fn compile_and_run_with_debug(filename: &str, debug: bool) -> Result<String, String> {
        // Check if file ends with .n extension
        if !filename.ends_with(".n") {
            return Err("Error: File must have .n extension".to_string());
        }

        // Read the file
        let source_code = match std::fs::read_to_string(filename) {
            Ok(content) => content,
            Err(err) => {
                return Err(format!("Error reading file '{}': {}", filename, err));
            }
        };

        if debug {
            println!("--- Source Code ---\n{}", source_code);
        }

        let mut lexer = Lexer::new(source_code);
        let tokens = lexer.tokenize();

        if debug {
            println!("--- Tokens ---");
            for token in &tokens {
                println!("{:?}", token);
            }
        }

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        if debug {
            println!("--- AST ---");
            // Assuming AST implements Debug
            println!("{:#?}", ast);
        }

        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast);

        if debug {
            println!("--- Bytecode ---\n");
            if bytecode.functions.len() > 0 {
                println!("--- Functions ---");
                for function in bytecode.functions.iter() {
                    println!("{}", function);
                }
            }
            if bytecode.constants.len() > 0 {
                println!("--- Constants ---");
                for constant in bytecode.constants.iter() {
                    println!("{}", constant);
                }
            }
            println!("--- Instructions ---");
            for instruction in bytecode.instructions.iter() {
                println!("{}", instruction);
            }
        }

        let mut vm = VirtualMachine::new(bytecode);

        if debug {
            println!("--- Runtime ---");
        }

        match vm.run() {
            Ok(()) => {
                vm.debug_stack();
                Ok("Successfully executed program".to_string())
            }
            Err(e) => {
                vm.debug_stack();
                Err(format!("Runtime error: {}", e))
            }
        }
    }
}

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file.n>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];

    match runtime::compile_and_run_with_debug(filename, true) {
        Ok(result) => {
            println!("=== EXECUTION ===");
            println!("{}", result);
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
