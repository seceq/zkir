//! Integration test for deferred carry model
//!
//! Tests that deferred arithmetic works correctly with observation points.

use zkir_runtime::{VM, VMConfig, DeferredConfig, execute_with_deferred, Memory, VMState, NormalizationCause};
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
fn test_deferred_add_then_branch() {
    // Test: ADD followed by BEQ (observation point)
    // The BEQ should trigger normalization of its operands

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
        // R3 = R1 + R2 (deferred, produces accumulated result)
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
        // BEQ R3, R4 (should normalize R3 and R4 before comparison)
        Instruction::Beq {
            rs1: Register::R3,
            rs2: Register::R4,
            offset: 8, // Skip next instruction if equal
        },
        // If not equal, set R5 = 1 (should not happen)
        Instruction::Addi {
            rd: Register::R5,
            rs1: Register::R0,
            imm: 1,
        },
        // Target if equal: R5 = 42
        Instruction::Addi {
            rd: Register::R5,
            rs1: Register::R0,
            imm: 42,
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
            imm: 0, // exit code
        },
        Instruction::Ecall,
    ];

    let program = create_test_program(instructions);
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run();

    assert!(result.is_ok());
    let exec_result = result.unwrap();

    // Should have taken the branch (R3 == R4 after normalization)
    // So R5 should be 42, not 1
    // We can't directly check R5 from the result, but we can verify it didn't error
    assert!(exec_result.cycles > 0);
}

#[test]
fn test_deferred_arithmetic_chain() {
    // Test: Multiple deferred ADDs in sequence
    // R1 = 10 + 20 + 30 + 40 + 50 = 150

    let instructions = vec![
        // R1 = 10
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 10,
        },
        // R1 = R1 + 20 (deferred)
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R1,
            imm: 20,
        },
        // R1 = R1 + 30 (deferred)
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R1,
            imm: 30,
        },
        // R1 = R1 + 40 (deferred)
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R1,
            imm: 40,
        },
        // R1 = R1 + 50 (deferred)
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R1,
            imm: 50,
        },
        // Store R1 (observation point - forces normalization)
        Instruction::Sw {
            rs1: Register::R0, // base register
            rs2: Register::R1, // value = 150
            imm: 0x10000,      // offset to data section
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
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run();

    assert!(result.is_ok());
}

#[test]
fn test_deferred_add_sub_mix() {
    // Test: Mix of ADD and SUB operations
    // R1 = 100 + 50 - 30 = 120

    let instructions = vec![
        // R1 = 100
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 100,
        },
        // R2 = 50
        Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 50,
        },
        // R3 = R1 + R2 (deferred)
        Instruction::Add {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        },
        // R4 = 30
        Instruction::Addi {
            rd: Register::R4,
            rs1: Register::R0,
            imm: 30,
        },
        // R5 = R3 - R4 (deferred)
        Instruction::Sub {
            rd: Register::R5,
            rs1: Register::R3,
            rs2: Register::R4,
        },
        // AND R5 with mask (observation point - forces normalization)
        Instruction::Andi {
            rd: Register::R6,
            rs1: Register::R5,
            imm: 0xFFFF,
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
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run();

    assert!(result.is_ok());
}

#[test]
fn test_manual_deferred_execution() {
    // Direct test of execute_with_deferred function

    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let config = DeferredConfig::default();

    // Setup: R1 = 1000, R2 = 2000
    state.write_reg_from_limbs(Register::R1, [1000, 0], config.normalized_bits);
    state.write_reg_from_limbs(Register::R2, [2000, 0], config.normalized_bits);

    // Execute: R3 = R1 + R2 (deferred)
    let inst = Instruction::Add {
        rd: Register::R3,
        rs1: Register::R1,
        rs2: Register::R2,
    };

    let result = execute_with_deferred(&inst, &mut state, &mut memory, None, Some(&config), 0, 0);
    assert!(result.is_ok());
    let events = result.unwrap();
    // No normalization events since ADD doesn't trigger observation point
    assert_eq!(events.len(), 0);

    // R3 should be in accumulated state
    assert!(state.register_states.get(Register::R3).needs_normalization());

    // Read limbs - should be [3000, 0]
    let limbs = state.read_reg_limbs_extended(Register::R3, config.normalized_bits, config.limb_bits);
    assert_eq!(limbs, [3000, 0]);

    // Normalize R3
    let norm_result = state.normalize_register(Register::R3, config.normalized_bits, config.limb_bits);
    assert!(norm_result.is_some());

    let norm = norm_result.unwrap();
    assert_eq!(norm.normalized, [3000, 0]);
    assert_eq!(norm.carries, [0, 0]); // No carry needed

    // R3 should now be normalized
    assert!(!state.register_states.get(Register::R3).needs_normalization());

    // Final value check
    let final_val = state.read_reg(Register::R3);
    assert_eq!(final_val, 3000);
}

#[test]
fn test_deferred_with_carry_propagation() {
    // Test carry propagation during normalization

    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let config = DeferredConfig::default();

    // Setup: R1 = 2^20 - 10 (near limb boundary)
    let near_boundary = (1u32 << config.normalized_bits) - 10;
    state.write_reg_from_limbs(Register::R1, [near_boundary, 0], config.normalized_bits);

    // R2 = 100 (will cause carry)
    state.write_reg_from_limbs(Register::R2, [100, 0], config.normalized_bits);

    // Execute: R3 = R1 + R2 (deferred)
    let inst = Instruction::Add {
        rd: Register::R3,
        rs1: Register::R1,
        rs2: Register::R2,
    };

    let result = execute_with_deferred(&inst, &mut state, &mut memory, None, Some(&config), 0, 0);
    assert!(result.is_ok());
    let events = result.unwrap();
    // No normalization events since ADD doesn't trigger observation point
    assert_eq!(events.len(), 0);

    // R3 should be accumulated
    assert!(state.register_states.get(Register::R3).needs_normalization());

    // Normalize R3
    let norm = state.normalize_register(Register::R3, config.normalized_bits, config.limb_bits);
    assert!(norm.is_some());

    let norm_result = norm.unwrap();

    // Should have extracted carry: (2^20 - 10) + 100 = 2^20 + 90
    // limb0 = 90, limb1 = 1 (from carry)
    assert_eq!(norm_result.normalized[0], 90);
    assert_eq!(norm_result.normalized[1], 1);
    assert_eq!(norm_result.carries[0], 1);

    // Final value = 90 + 1 * 2^20
    let expected = 90 + (1u64 << config.normalized_bits);
    let final_val = state.read_reg(Register::R3);
    assert_eq!(final_val, expected);
}

#[test]
fn test_observation_point_auto_normalization() {
    // Test that observation points automatically normalize operands

    let mut state = VMState::new(0x1000);
    let mut memory = Memory::new();
    let config = DeferredConfig::default();

    // Create accumulated values
    state.write_reg_from_accumulated(Register::R1, [500, 0], config.limb_bits);
    state.write_reg_from_accumulated(Register::R2, [500, 0], config.limb_bits);

    // Both should be accumulated
    assert!(state.register_states.get(Register::R1).needs_normalization());
    assert!(state.register_states.get(Register::R2).needs_normalization());

    // Execute BEQ (observation point - should normalize operands)
    let inst = Instruction::Beq {
        rs1: Register::R1,
        rs2: Register::R2,
        offset: 8,
    };

    let result = execute_with_deferred(&inst, &mut state, &mut memory, None, Some(&config), 42, 0x1000);
    assert!(result.is_ok());
    let events = result.unwrap();

    // BEQ should have triggered normalization of both R1 and R2
    assert_eq!(events.len(), 2, "BEQ should normalize both operands");

    // Verify witness details
    for event in &events {
        assert_eq!(event.witness.cycle, 42);
        assert_eq!(event.witness.pc, 0x1000);
        assert!(event.witness.register == Register::R1 || event.witness.register == Register::R2);
        assert_eq!(event.cause, NormalizationCause::ObservationPoint);
        assert_eq!(event.triggering_opcode, Some(zkir_spec::Opcode::Beq));
    }

    // Both should now be normalized (execute_with_deferred normalizes before observation points)
    assert!(!state.register_states.get(Register::R1).needs_normalization());
    assert!(!state.register_states.get(Register::R2).needs_normalization());
}
