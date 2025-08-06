use std::io::{self, Write};

pub fn print_intro() {
    let intro = r#"
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║    ███╗   ███╗██╗██████╗ ██████╗  ██████╗ ██╗    ██╗                         ║
║    ████╗ ████║██║██╔══██╗██╔══██╗██╔═══██╗██║    ██║                         ║
║    ██╔████╔██║██║██████╔╝██████╔╝██║   ██║██║ █╗ ██║                         ║
║    ██║╚██╔╝██║██║██╔══██╗██╔══██╗██║   ██║██║███╗██║                         ║
║    ██║ ╚═╝ ██║██║██║  ██║██║  ██║╚██████╔╝╚███╔███╔╝                         ║
║    ╚═╝     ╚═╝╚═╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝                          ║
║                                                                              ║
║                        🌊 The Reflective Language 🌊                         ║
║                                                                              ║
║  "Code flows like verse, swift and bright,                                   ║
║   Bytecode dances in memory's light."                                        ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  Welcome to Mirrow - where reflection meets performance!                     ║
║                                                                              ║
║  Features:                                                                   ║
║  • 🔄 Async/await support                                                    ║
║  • 🏗️  Pattern matching with enums                                           ║
║  • 📦 Module system with imports                                             ║
║  • ⚡ Bytecode compilation                                                    ║
║  • 🎯 Expression-based syntax                                                ║
║                                                                              ║
║  Usage:                                                                      ║
║    mirrow <file.mir>     - Run a Mirrow file                                 ║
║    mirrow --help         - Show help information                             ║
║    mirrow --version      - Show version information                          ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
"#;

    print!("{}", intro);
    io::stdout().flush().unwrap();
}

pub fn print_help() {
    println!("Mirrow - The Reflective Language");
    println!();
    println!("USAGE:");
    println!("    mirrow [OPTIONS] [FILE]");
    println!();
    println!("ARGS:");
    println!("    <FILE>    The .mir file to execute");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -V, --version    Print version information");
    println!("    --debug          Enable debug output");
    println!();
    println!("EXAMPLES:");
    println!("    mirrow main.mir                 # Run main.mir");
    println!("    mirrow --debug example.mir      # Run with debug output");
}

pub fn print_version() {
    println!("mirrow {}", env!("CARGO_PKG_VERSION"));
    println!("🌊 The Reflective Language");
}

pub fn print_error(message: &str) {
    eprintln!("❌ Error: {}", message);
}

pub fn print_success(message: &str) {
    println!("✅ {}", message);
}
