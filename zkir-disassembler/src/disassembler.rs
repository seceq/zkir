//! Main disassembler logic for ZKIR v3.4

use zkir_spec::Program;
use crate::error::Result;
use crate::decoder::decode;
use crate::formatter::format;

/// Disassemble a program into assembly text
pub fn disassemble(program: &Program) -> Result<String> {
    let mut output = String::new();

    // Header
    output.push_str("; ZKIR v3.4 Disassembly\n");
    output.push_str(";\n");

    // Configuration
    let config = program.config();
    output.push_str(&std::format!("; Configuration:\n"));
    output.push_str(&std::format!(";   Limb bits:  {}\n", config.limb_bits));
    output.push_str(&std::format!(";   Data limbs: {} ({}-bit values)\n", config.data_limbs, config.data_bits()));
    output.push_str(&std::format!(";   Addr limbs: {} ({}-bit addresses)\n", config.addr_limbs, config.addr_bits()));
    output.push_str(";\n");

    // Program info
    output.push_str(&std::format!("; Entry point: 0x{:08X}\n", program.header.entry_point));
    output.push_str(&std::format!("; Code size:   {} bytes ({} instructions)\n",
        program.header.code_size, program.code.len()));
    output.push_str(&std::format!("; Data size:   {} bytes\n", program.header.data_size));
    output.push_str("\n");

    let mut addr = program.header.entry_point;

    for &word in &program.code {
        // Address label
        output.push_str(&std::format!("0x{:08X}:  ", addr));

        // Hex encoding
        output.push_str(&std::format!("{:08X}  ", word));

        // Decode and format
        match decode(word) {
            Ok(instr) => {
                output.push_str(&format(&instr));
            }
            Err(e) => {
                output.push_str(&std::format!("; ERROR: {}", e));
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
    use zkir_spec::Config;

    #[test]
    fn test_disassemble_simple() {
        use zkir_spec::Opcode;
        let code = vec![
            Opcode::Ecall.to_u8() as u32,   // ecall = 0x50
            Opcode::Ebreak.to_u8() as u32,  // ebreak = 0x51
        ];

        let mut program = Program::new();
        program.code = code;
        program.header.code_size = 8;

        let asm = disassemble(&program).unwrap();

        assert!(asm.contains("ecall"));
        assert!(asm.contains("ebreak"));
        assert!(asm.contains("ZKIR v3.4"));
    }

    #[test]
    fn test_disassemble_with_config() {
        use zkir_spec::Opcode;
        let code = vec![Opcode::Ecall.to_u8() as u32];  // ecall = 0x50

        let config = Config {
            limb_bits: 20,
            data_limbs: 2,
            addr_limbs: 2,
        };

        let mut program = Program::with_config(config).unwrap();
        program.code = code;
        program.header.code_size = 4;

        let asm = disassemble(&program).unwrap();

        assert!(asm.contains("Limb bits:  20"));
        assert!(asm.contains("Data limbs: 2"));
        assert!(asm.contains("40-bit"));
    }
}
