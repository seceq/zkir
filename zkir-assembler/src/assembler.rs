//! Main assembler logic with label resolution and directive support

use zkir_spec::{Program, Instruction, CODE_BASE};
use crate::error::{Result, AssemblerError};
use crate::parser::parse_instruction;
use crate::encoder::encode;
use std::collections::HashMap;

/// Assembly line type
#[derive(Debug, Clone)]
enum Line {
    /// Label definition (name)
    Label(String),
    /// Instruction
    Instruction(Instruction),
    /// Directive with arguments
    Directive(String, Vec<String>),
    /// Empty line or comment
    Empty,
}

/// Assemble source code into a program
///
/// Supports:
/// - Instructions (all 77 v2.2 instructions)
/// - Labels for branch/jump targets
/// - Comments (# and ; style)
/// - Directives (.text, .data - currently parsed but not fully implemented)
///
/// # Example
/// ```
/// use zkir_assembler::assemble;
///
/// let source = r#"
///     .text
/// main:
///     addi a0, zero, 10
///     addi a1, zero, 32
///     add a2, a0, a1
///     write a2
///     halt
/// "#;
///
/// let program = assemble(source).unwrap();
/// ```
pub fn assemble(source: &str) -> Result<Program> {
    // First pass: parse all lines and collect labels
    let (lines, labels) = first_pass(source)?;

    // Second pass: encode instructions with resolved labels
    let code = second_pass(&lines, &labels)?;

    Ok(Program::new(code))
}

/// First pass: parse lines and collect label addresses
fn first_pass(source: &str) -> Result<(Vec<Line>, HashMap<String, u32>)> {
    let mut lines = Vec::new();
    let mut labels = HashMap::new();
    let mut pc = CODE_BASE;

    for (line_num, line_text) in source.lines().enumerate() {
        let line_text = line_text.trim();

        // Skip empty lines and pure comments
        if line_text.is_empty() || line_text.starts_with(';') || line_text.starts_with('#') {
            lines.push(Line::Empty);
            continue;
        }

        // Check for label (ends with :)
        if let Some(label_end) = line_text.find(':') {
            let label = line_text[..label_end].trim().to_string();

            // Validate label name
            if !is_valid_label(&label) {
                return Err(AssemblerError::SyntaxError {
                    line: line_num + 1,
                    column: 0,
                    message: format!("Invalid label name: {}", label),
                });
            }

            // Check for duplicate
            if labels.contains_key(&label) {
                return Err(AssemblerError::SyntaxError {
                    line: line_num + 1,
                    column: 0,
                    message: format!("Duplicate label: {}", label),
                });
            }

            labels.insert(label.clone(), pc);
            lines.push(Line::Label(label));

            // Check if there's an instruction on the same line after the colon
            let rest = line_text[label_end + 1..].trim();
            if !rest.is_empty() && !rest.starts_with(';') && !rest.starts_with('#') {
                // Parse instruction on same line as label
                match parse_instruction(rest) {
                    Ok(instr) => {
                        lines.push(Line::Instruction(instr));
                        pc += 4;
                    }
                    Err(e) => {
                        return Err(AssemblerError::SyntaxError {
                            line: line_num + 1,
                            column: label_end + 1,
                            message: format!("Failed to parse instruction: {}", e),
                        });
                    }
                }
            }
            continue;
        }

        // Check for directive (starts with .)
        if line_text.starts_with('.') {
            let parts: Vec<&str> = line_text.split_whitespace().collect();
            if parts.is_empty() {
                lines.push(Line::Empty);
                continue;
            }

            let directive = parts[0][1..].to_string(); // Remove leading '.'
            let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

            lines.push(Line::Directive(directive, args));
            // Directives don't consume PC space
            continue;
        }

        // Parse as instruction
        match parse_instruction(line_text) {
            Ok(instr) => {
                lines.push(Line::Instruction(instr));
                pc += 4;
            }
            Err(e) => {
                return Err(AssemblerError::SyntaxError {
                    line: line_num + 1,
                    column: 0,
                    message: format!("Failed to parse instruction: {}", e),
                });
            }
        }
    }

    Ok((lines, labels))
}

/// Second pass: encode instructions with resolved labels
fn second_pass(lines: &[Line], _labels: &HashMap<String, u32>) -> Result<Vec<u32>> {
    let mut code = Vec::new();

    for line in lines.iter() {
        match line {
            Line::Instruction(instr) => {
                // For now, encode as-is. Label resolution in immediates would require
                // instruction modification, which we'll handle in future enhancement
                let encoded = encode(instr);
                code.push(encoded);
            }
            Line::Label(_) | Line::Directive(_, _) | Line::Empty => {
                // Labels and directives don't generate code
            }
        }
    }

    Ok(code)
}

/// Check if a label name is valid
fn is_valid_label(label: &str) -> bool {
    if label.is_empty() {
        return false;
    }

    // First character must be letter or underscore
    let first = label.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Rest can be alphanumeric or underscore
    label.chars().all(|c| c.is_alphanumeric() || c == '_')
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

    #[test]
    fn test_assemble_with_labels() {
        let source = r#"
            ; Test with labels (using numeric offsets for now)
        start:
            addi a0, zero, 10
            beq a0, zero, 8
            addi a1, zero, 20
        end:
            halt
        "#;

        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 4);
    }

    #[test]
    fn test_assemble_label_on_same_line() {
        let source = r#"
        start: addi a0, zero, 42
        end: halt
        "#;

        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 2);
    }

    #[test]
    fn test_assemble_with_directive() {
        let source = r#"
            .text
            addi a0, zero, 10
            halt
        "#;

        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 2);
    }

    #[test]
    fn test_assemble_comments() {
        let source = r#"
            # Hash comment
            addi a0, zero, 10  ; Inline semicolon comment
            ; Full line semicolon comment
            halt # Another comment
        "#;

        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 2);
    }

    #[test]
    fn test_assemble_all_instruction_types() {
        let source = r#"
            ; R-type
            add a0, a1, a2
            sub t0, t1, t2

            ; I-type
            addi a0, a1, 100
            lw a0, 16(sp)

            ; S-type
            sw a0, 16(sp)

            ; B-type (using numeric offset)
        loop:
            beq a0, a1, 0

            ; J-type
            jal ra, 1000

            ; U-type
            lui a0, 0x1234

            ; ZK ops
            read a0
            write a1

            ; System
            ecall
            halt
        "#;

        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 12); // Labels don't generate code
    }

    #[test]
    fn test_invalid_label() {
        let source = r#"
        123invalid:
            halt
        "#;

        let result = assemble(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_label() {
        let source = r#"
        start:
            addi a0, zero, 10
        start:
            halt
        "#;

        let result = assemble(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_program() {
        let source = r#"
            ; Just comments
            # And more comments
        "#;

        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 0);
    }

    #[test]
    fn test_label_collection() {
        let source = r#"
        start:
            addi a0, zero, 10
        middle:
            addi a1, zero, 20
        end:
            halt
        "#;

        let (_lines, labels) = first_pass(source).unwrap();

        assert_eq!(labels.len(), 3);
        assert_eq!(labels.get("start"), Some(&CODE_BASE));
        assert_eq!(labels.get("middle"), Some(&(CODE_BASE + 4)));
        assert_eq!(labels.get("end"), Some(&(CODE_BASE + 8)));
    }
}
