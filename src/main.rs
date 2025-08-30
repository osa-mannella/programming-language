mod compiler;
mod debug;
mod interpreter;
mod lexer;
mod parser;
mod types;

use compiler::Compiler;
use interpreter::VirtualMachine;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::process;

use crate::debug::print_tokens;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file.n>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];

    // Check if file ends with .n extension
    if !filename.ends_with(".n") {
        eprintln!("Error: File must have .n extension");
        process::exit(1);
    }

    // Read the file
    let source_code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    println!("=== n Parser ===");
    println!("File: {}", filename);
    println!();

    let mut lexer = Lexer::new(source_code);
    let tokens = lexer.tokenize();

    print_tokens(&tokens);

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    println!("=== AST ===");
    println!("{:#?}", ast);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast);

    println!();
    println!("{}", bytecode);

    println!("=== EXECUTION ===");
    let mut vm = VirtualMachine::new(bytecode);
    match vm.run() {
        Ok(()) => {
            println!("Program executed successfully");
            vm.debug_stack();
        }
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            vm.debug_stack();
            process::exit(1);
        }
    }
}
