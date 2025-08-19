use crate::library::compiler::{BytecodeProgram, ConstantValue};

fn get_constant_string(bytecode: &BytecodeProgram, index: u16) -> String {
    if let Some(constant) = bytecode.constants.get(index as usize) {
        match &constant.value {
            ConstantValue::String(s) => s.clone(),
            ConstantValue::Number(n) => n.to_string(),
            ConstantValue::Boolean(b) => b.to_string(),
            ConstantValue::FunctionRef(f) => format!("func_{}", f),
        }
    } else {
        format!("INVALID_CONST_{}", index)
    }
}

pub fn print_bytecode_debug(bytecode: &BytecodeProgram) {
    println!("=== MIRROW BYTECODE DEBUG OUTPUT ===");

    // Header
    println!("\n--- HEADER ---");
    println!(
        "Magic: {:?}",
        std::str::from_utf8(&bytecode.header.magic).unwrap_or("Invalid")
    );
    println!("Version: {}", bytecode.header.version);
    println!("Flags: {}", bytecode.header.flags);
    println!("Message: {}", bytecode.header.message);

    // Constants
    println!("\n--- CONSTANTS ({}) ---", bytecode.constants.len());
    for (i, constant) in bytecode.constants.iter().enumerate() {
        match &constant.value {
            ConstantValue::String(s) => println!("{}: String(\"{}\")", i, s),
            ConstantValue::Number(n) => println!("{}: Number({})", i, n),
            ConstantValue::Boolean(b) => println!("{}: Boolean({})", i, b),
            ConstantValue::FunctionRef(f) => println!("{}: FunctionRef({})", i, f),
        }
    }

    // Functions
    println!("\n--- FUNCTIONS ({}) ---", bytecode.functions.len());
    for (i, function) in bytecode.functions.iter().enumerate() {
        println!(
            "{}: args={}, locals={}, offset={}",
            i, function.arg_count, function.local_count, function.offset
        );
    }

    // Enums
    println!("\n--- ENUMS ({}) ---", bytecode.enums.len());
    for (i, enum_def) in bytecode.enums.iter().enumerate() {
        let enum_name = get_constant_string(bytecode, enum_def.name_index);
        println!(
            "{}: {} (name_index={}, variants={})",
            i,
            enum_name,
            enum_def.name_index,
            enum_def.variants.len()
        );
        for (j, variant) in enum_def.variants.iter().enumerate() {
            let variant_name = get_constant_string(bytecode, variant.name_index);
            println!(
                "  {}: {} (name_index={}, fields={})",
                j, variant_name, variant.name_index, variant.field_count
            );
        }
    }

    // Opcode mapping
    println!("\n--- OPCODE MAP ---");
    let mut sorted_opcodes: Vec<_> = bytecode.opcode_map.iter().collect();
    sorted_opcodes.sort_by_key(|&(_, opcode)| opcode);
    for (name, &opcode) in sorted_opcodes {
        println!("{:#04X}: {}", opcode, name);
    }

    // Instructions
    println!(
        "\n--- INSTRUCTIONS ({} bytes) ---",
        bytecode.instructions.len()
    );
    if bytecode.instructions.is_empty() {
        println!("(No instructions generated)");
    } else {
        let mut i = 0;
        while i < bytecode.instructions.len() {
            print!("{:04}: ", i);

            let opcode = bytecode.instructions[i];
            let opcode_name = bytecode
                .opcode_map
                .iter()
                .find(|&(_, op)| *op == opcode)
                .map(|(name, _)| name.as_str())
                .unwrap_or("UNKNOWN");

            print!("{:#04X} {} ", opcode, opcode_name);

            // Try to decode operands based on known instruction formats
            match opcode_name {
                "load_const" | "load_global" | "store_global" | "jump" | "jump_if_false"
                | "jump_if_true" => {
                    if i + 2 < bytecode.instructions.len() {
                        let operand = u16::from_le_bytes([
                            bytecode.instructions[i + 1],
                            bytecode.instructions[i + 2],
                        ]);
                        println!("{}", operand);
                        i += 3;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "load_local" => {
                    if i + 1 < bytecode.instructions.len() {
                        println!("{}", bytecode.instructions[i + 1]);
                        i += 2;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "store_local" => {
                    if i + 2 < bytecode.instructions.len() {
                        let operand = u16::from_le_bytes([
                            bytecode.instructions[i + 1],
                            bytecode.instructions[i + 2],
                        ]);
                        println!("{}", operand);
                        i += 3;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "call" | "call_global" => {
                    if i + 2 < bytecode.instructions.len() {
                        println!(
                            "func={}, argc={}",
                            bytecode.instructions[i + 1],
                            bytecode.instructions[i + 2]
                        );
                        i += 3;
                    } else {
                        println!("(incomplete operands)");
                        i += 1;
                    }
                }
                "call_native" => {
                    if i + 2 < bytecode.instructions.len() {
                        println!(
                            "native_id={}, argc={}",
                            bytecode.instructions[i + 1],
                            bytecode.instructions[i + 2]
                        );
                        i += 3;
                    } else {
                        println!("(incomplete operands)");
                        i += 1;
                    }
                }
                "create_enum" => {
                    if i + 3 < bytecode.instructions.len() {
                        let enum_index = bytecode.instructions[i + 1];
                        let variant_index = bytecode.instructions[i + 2];
                        let field_count = bytecode.instructions[i + 3];

                        // Try to resolve enum and variant names for better readability
                        let mut debug_info = format!(
                            "enum={}, variant={}, fields={}",
                            enum_index, variant_index, field_count
                        );
                        if let Some(enum_def) = bytecode.enums.get(enum_index as usize) {
                            let enum_name = get_constant_string(bytecode, enum_def.name_index);
                            if let Some(variant) = enum_def.variants.get(variant_index as usize) {
                                let variant_name =
                                    get_constant_string(bytecode, variant.name_index);
                                debug_info = format!(
                                    "{}::{} (enum={}, variant={}, fields={})",
                                    enum_name, variant_name, enum_index, variant_index, field_count
                                );
                            }
                        }
                        println!("{}", debug_info);
                        i += 4;
                    } else {
                        println!("(incomplete operands)");
                        i += 1;
                    }
                }
                "get_enum_variant" => {
                    if i + 1 < bytecode.instructions.len() {
                        println!("enum_index={}", bytecode.instructions[i + 1]);
                        i += 2;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "match_literal" => {
                    if i + 2 < bytecode.instructions.len() {
                        let const_index = u16::from_le_bytes([
                            bytecode.instructions[i + 1],
                            bytecode.instructions[i + 2],
                        ]);
                        let const_value = get_constant_string(bytecode, const_index);
                        println!("const_index={} (value={})", const_index, const_value);
                        i += 3;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "match_enum_variant" => {
                    if i + 2 < bytecode.instructions.len() {
                        let enum_index = bytecode.instructions[i + 1];
                        let variant_index = bytecode.instructions[i + 2];

                        // Try to resolve enum and variant names for better readability
                        let mut debug_info =
                            format!("enum={}, variant={}", enum_index, variant_index);
                        if let Some(enum_def) = bytecode.enums.get(enum_index as usize) {
                            let enum_name = get_constant_string(bytecode, enum_def.name_index);
                            if let Some(variant) = enum_def.variants.get(variant_index as usize) {
                                let variant_name =
                                    get_constant_string(bytecode, variant.name_index);
                                debug_info = format!(
                                    "{}::{} (enum={}, variant={})",
                                    enum_name, variant_name, enum_index, variant_index
                                );
                            }
                        }
                        println!("{}", debug_info);
                        i += 3;
                    } else {
                        println!("(incomplete operands)");
                        i += 1;
                    }
                }
                "extract_enum_field" => {
                    if i + 1 < bytecode.instructions.len() {
                        println!("field_index={}", bytecode.instructions[i + 1]);
                        i += 2;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "string_concat" => {
                    if i + 1 < bytecode.instructions.len() {
                        println!("parts={}", bytecode.instructions[i + 1]);
                        i += 2;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "create_array" => {
                    if i + 1 < bytecode.instructions.len() {
                        println!("element_count={}", bytecode.instructions[i + 1]);
                        i += 2;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "array_append" => {
                    if i + 1 < bytecode.instructions.len() {
                        println!("element_count={}", bytecode.instructions[i + 1]);
                        i += 2;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "struct_create" => {
                    if i + 1 < bytecode.instructions.len() {
                        println!("field_count={}", bytecode.instructions[i + 1]);
                        i += 2;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "match_struct_fields" => {
                    if i + 1 < bytecode.instructions.len() {
                        println!("field_count={}", bytecode.instructions[i + 1]);
                        i += 2;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "extract_struct_field" => {
                    println!(""); // No additional operands (field name is on stack)
                    i += 1;
                }
                "call_indirect" => {
                    if i + 1 < bytecode.instructions.len() {
                        println!("argc={}", bytecode.instructions[i + 1]);
                        i += 2;
                    } else {
                        println!("(incomplete operand)");
                        i += 1;
                    }
                }
                "add" | "sub" | "mul" | "div" | "power" | "equal" | "not_equal" | "less"
                | "greater" | "less_equal" | "greater_equal" | "and" | "or" | "pop" | "dup"
                | "return" | "halt" | "match_fail" | "index_access" | "get_type" => {
                    println!(""); // No operands
                    i += 1;
                }
                _ => {
                    println!(""); // Unknown instruction format
                    i += 1;
                }
            }
        }
    }

    println!("\n=== END BYTECODE DEBUG ===");
}
