mod library;

use library::cli::*;
use library::compiler::compile_program;
use library::debug::print_bytecode_debug;
use library::lexer::Lexer;
use library::parser::Parser;
use std::env;
use std::fs;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    match args.len() {
        1 => {
            print_intro();
            println!();
            print_help();
            exit(0);
        }
        2 => {
            let arg = &args[1];
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    exit(0);
                }
                "-V" | "--version" => {
                    print_version();
                    exit(0);
                }
                filename => {
                    print_intro();
                    println!();
                    run_file(filename);
                }
            }
        }
        3 => {
            let flag = &args[1];
            let filename = &args[2];
            
            match flag.as_str() {
                "--debug" => {
                    print_intro();
                    println!();
                    println!("🔍 Debug mode enabled");
                    println!();
                    run_file(filename);
                }
                "--compile-only" => {
                    print_intro();
                    println!();
                    println!("🔧 Compile-only mode");
                    println!();
                    compile_only(filename);
                }
                _ => {
                    print_error(&format!("Unknown flag: {}", flag));
                    print_help();
                    exit(1);
                }
            }
        }
        _ => {
            print_error("Too many arguments");
            print_help();
            exit(1);
        }
    }
}

fn run_file(filename: &str) {
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            print_error(&format!("Could not open file \"{}\": {}", filename, e));
            exit(1);
        }
    };
    
    print_success(&format!("Compiling {}", filename));
    
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    if parser.had_error {
        print_error("Parse errors encountered");
        exit(1);
    }
    
    let bytecode = match compile_program(program) {
        Ok(bytecode) => bytecode,
        Err(e) => {
            print_error(&format!("Compilation failed: {}", e));
            exit(1);
        }
    };
    print_success("Compilation complete!");
    println!();
    print_bytecode_debug(&bytecode);
}

fn compile_only(filename: &str) {
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            print_error(&format!("Could not open file \"{}\": {}", filename, e));
            exit(1);
        }
    };
    
    print_success(&format!("Compiling {} (compile-only mode)", filename));
    
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    if parser.had_error {
        print_error("Parse errors encountered");
        exit(1);
    }
    
    let _bytecode = match compile_program(program) {
        Ok(bytecode) => bytecode,
        Err(e) => {
            print_error(&format!("Compilation failed: {}", e));
            exit(1);
        }
    };
    print_success("Compilation successful! ✨");
}
