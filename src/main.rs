mod lib;
use lib::lexer::Lexer;
use lib::parser::Parser;
use std::env;
use std::fs;
use std::process::exit;

fn main() {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        exit(1);
    }

    // Read the source file
    let filename = &args[1];
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not open file \"{}\": {}", filename, e);
            exit(1);
        }
    };

    // Initialize lexer
    let lexer = Lexer::new(&source);

    // Initialize parser
    let mut parser = Parser::new(lexer);

    // Parse the program
    let program = parser.parse_program();

    //println!("{:?}", program);
    // Print AST
    program.print();

    // (No need to manually free memory—Rust handles it)
}
