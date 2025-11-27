//! Main disassembler logic

use zkir_spec::Program;
use crate::error::Result;
use crate::decoder::decode;
use crate::formatter::format;

/// Disassemble a program into assembly text
pub fn disassemble(program: &Program) -> Result<String> {
    let mut output = String::new();

    output.push_str("; ZK IR Disassembly\n");
    output.push_str(&format!("; Entry point: 0x{:08X}\n", program.header.entry_point));
    output.push_str(&format!("; Code size: {} bytes ({} instructions)\n",
        program.header.code_size, program.code.len()));
    output.push_str("\n");

    let mut addr = program.header.entry_point;

    for &word in &program.code {
        // Address label
        output.push_str(&format!("0x{:08X}:  ", addr));

        // Hex encoding
        output.push_str(&format!("{:08X}  ", word));

        // Decode and format
        match decode(word) {
            Ok(instr) => {
                output.push_str(&format(&instr));
            }
            Err(e) => {
                output.push_str(&format!("; ERROR: {}", e));
            }
        }

        output.push('\n');
        addr += 4;
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_simple() {
        let code = vec![
            0x00000073, // ecall
            0x00100073, // ebreak
        ];

        let program = Program::new(code);
        let asm = disassemble(&program).unwrap();

        assert!(asm.contains("ecall"));
        assert!(asm.contains("ebreak"));
    }
}
