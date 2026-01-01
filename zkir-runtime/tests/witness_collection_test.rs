//! Test normalization witness collection in VM execution
//!
//! Verifies that the VM correctly collects normalization witnesses
//! during execution of programs using the deferred carry model.

use zkir_runtime::{VM, VMConfig};
use zkir_spec::{Program, Instruction, Register};
use zkir_assembler;

fn create_test_program(instructions: Vec<Instruction>) -> Program {
    let mut program = Program::new();
    let code: Vec<u32> = instructions
        .iter()
        .map(|inst| zkir_assembler::encode(inst))
        .collect();
    program.code = code;
    program.header.code_size = (program.code.len() * 4) as u32;
    program
}

#[test]
fn test_witness_collection_with_deferred_model() {
    // Program that uses deferred ADD followed by BEQ (observation point)
    let instructions = vec![
        // R1 = 100
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 100,
        },
        // R2 = 200
        Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 200,
        },
        // R3 = R1 + R2 (deferred - produces accumulated result)
        Instruction::Add {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        },
        // R4 = 300 (for comparison)
        Instruction::Addi {
            rd: Register::R4,
            rs1: Register::R0,
            imm: 300,
        },
        // BEQ R3, R4 (observation point - normalizes R3 and R4)
        Instruction::Beq {
            rs1: Register::R3,
            rs2: Register::R4,
            offset: 8,
        },
        // Should skip this
        Instruction::Addi {
            rd: Register::R5,
            rs1: Register::R0,
            imm: 999,
        },
        // Exit
        Instruction::Addi {
            rd: Register::R10,
            rs1: Register::R0,
            imm: 0, // SYSCALL_EXIT
        },
        Instruction::Addi {
            rd: Register::R11,
            rs1: Register::R0,
            imm: 0,
        },
        Instruction::Ecall,
    ];

    let program = create_test_program(instructions);

    // Run with deferred model DISABLED - should have NO witnesses
    let mut config_no_deferred = VMConfig::default();
    config_no_deferred.enable_deferred_model = false;
    let vm_no_deferred = VM::new(program.clone(), vec![], config_no_deferred);
    let result_no_deferred = vm_no_deferred.run().unwrap();

    assert_eq!(result_no_deferred.normalization_witnesses.len(), 0,
        "Without deferred model, no normalization witnesses should be collected");

    // Run with deferred model ENABLED - should collect witnesses
    let mut config_with_deferred = VMConfig::default();
    config_with_deferred.enable_deferred_model = true;
    let vm_with_deferred = VM::new(program, vec![], config_with_deferred);
    let result_with_deferred = vm_with_deferred.run().unwrap();

    // Should have normalization witnesses from the BEQ observation point
    assert!(result_with_deferred.normalization_witnesses.len() > 0,
        "With deferred model, normalization witnesses should be collected");

    // Verify witness details
    for event in &result_with_deferred.normalization_witnesses {
        // All witnesses should be from observation points (BEQ)
        assert_eq!(event.cause, zkir_runtime::NormalizationCause::ObservationPoint,
            "All witnesses should be from observation points");

        // Should have triggering opcode
        assert!(event.triggering_opcode.is_some(),
            "Observation point witnesses should have triggering opcode");

        // Verify witness is valid
        assert!(event.witness.verify(),
            "Normalization witness should be mathematically correct");
    }
}

#[test]
fn test_witness_collection_with_carry_propagation() {
    // Program that triggers carry extraction
    let instructions = vec![
        // R1 = 2^20 - 10 (near limb boundary)
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: (1 << 20) - 10,
        },
        // R2 = 100 (will cause carry)
        Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 100,
        },
        // R3 = R1 + R2 (deferred - will need carry extraction)
        Instruction::Add {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        },
        // Store R3 (observation point - normalizes R3 with carry)
        Instruction::Addi {
            rd: Register::R4,
            rs1: Register::R0,
            imm: 0x10000,
        },
        Instruction::Sw {
            rs1: Register::R4,
            rs2: Register::R3,
            imm: 0,
        },
        // Exit
        Instruction::Addi {
            rd: Register::R10,
            rs1: Register::R0,
            imm: 0,
        },
        Instruction::Addi {
            rd: Register::R11,
            rs1: Register::R0,
            imm: 0,
        },
        Instruction::Ecall,
    ];

    let program = create_test_program(instructions);

    // Run with deferred model enabled
    let mut config = VMConfig::default();
    config.enable_deferred_model = true;
    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    // Should have normalization witnesses
    assert!(result.normalization_witnesses.len() > 0,
        "Should collect normalization witnesses");

    // At least one witness should have carries extracted
    let has_carries = result.normalization_witnesses.iter()
        .any(|event| event.witness.has_carries());

    assert!(has_carries,
        "At least one normalization should have extracted carries");

    // Verify all witnesses are valid
    for event in &result.normalization_witnesses {
        assert!(event.witness.verify(),
            "All normalization witnesses should be mathematically correct");
    }
}

#[test]
fn test_witness_cycle_and_pc_tracking() {
    // Simple program to verify cycle/PC tracking in witnesses
    let instructions = vec![
        Instruction::Addi { rd: Register::R1, rs1: Register::R0, imm: 10 },
        Instruction::Addi { rd: Register::R2, rs1: Register::R0, imm: 20 },
        Instruction::Add { rd: Register::R3, rs1: Register::R1, rs2: Register::R2 },
        Instruction::Beq { rs1: Register::R3, rs2: Register::R3, offset: 4 }, // Cycle 3
        Instruction::Ebreak,
    ];

    let program = create_test_program(instructions);

    let mut config = VMConfig::default();
    config.enable_deferred_model = true;
    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    // Should have witnesses from the BEQ at cycle 3
    assert!(result.normalization_witnesses.len() > 0);

    for event in &result.normalization_witnesses {
        // Witnesses should have cycle >= 3 (when BEQ executes)
        assert!(event.witness.cycle >= 3,
            "Witness cycle should be >= 3 (when BEQ executes)");

        // PC should be valid code address
        assert!(event.witness.pc >= 0x1000,
            "Witness PC should be in code section");
    }
}
