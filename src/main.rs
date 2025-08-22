mod types;
mod lexer;
mod debug;
mod parser;
mod compiler;

use std::env;
use std::fs;
use std::process;
use lexer::Lexer;
use debug::print_tokens;
use parser::Parser;
use compiler::Compiler;

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
}
