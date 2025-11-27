//! Main assembler logic

use zkir_spec::Program;
use crate::error::Result;
use crate::parser::parse_instruction;
use crate::encoder::encode;

/// Assemble source code into a program
pub fn assemble(source: &str) -> Result<Program> {
    let mut instructions = Vec::new();

    // Parse each line
    for (line_num, line) in source.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        // Parse instruction
        match parse_instruction(line) {
            Ok(instr) => {
                let encoded = encode(&instr);
                instructions.push(encoded);
            }
            Err(e) => {
                eprintln!("Error on line {}: {}", line_num + 1, e);
                return Err(e);
            }
        }
    }

    Ok(Program::new(instructions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_simple() {
        let source = r#"
            ; Simple test
            ecall
            halt
        "#;

        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 2);
    }
}
