//! Virtual Machine for ZK IR v2.2

use zkir_spec::Program;
use zkir_disassembler::decoder::decode;
use crate::state::{VMState, HaltReason};
use crate::io::IOHandler;
use crate::error::RuntimeError;
use crate::execute::execute;

#[derive(Debug, Clone)]
pub struct VMConfig {
    pub max_cycles: u64,
    pub stack_size: u32,
    pub heap_size: u32,
    pub trace_enabled: bool,
}

impl Default for VMConfig {
    fn default() -> Self {
        VMConfig {
            max_cycles: 10_000_000,
            stack_size: 1 << 20,
            heap_size: 1 << 20,
            trace_enabled: false,
        }
    }
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub outputs: Vec<u32>,
    pub commitments: Vec<u32>,
    pub cycles: u64,
    pub halt_reason: HaltReason,
}

pub struct VM {
    state: VMState,
    io: IOHandler,
    config: VMConfig,
}

impl VM {
    pub fn new(program: Program, inputs: Vec<u32>, config: VMConfig) -> Self {
        let mut state = VMState::new(config.stack_size, config.heap_size);
        state.memory.load_code(&program.code);
        if !program.data.is_empty() {
            state.memory.load_data(&program.data);
        }
        state.pc = program.header.entry_point;

        VM {
            state,
            io: IOHandler::new(inputs),
            config,
        }
    }

    pub fn run(mut self) -> Result<ExecutionResult, RuntimeError> {
        loop {
            // Check halt condition
            if self.state.halted {
                break;
            }

            // Check cycle limit
            if self.state.cycle >= self.config.max_cycles {
                self.state.halt(HaltReason::OutOfCycles);
                break;
            }

            // Fetch instruction word from memory
            let word = match self.state.memory.load_word(self.state.pc, self.state.cycle) {
                Ok(w) => w,
                Err(reason) => {
                    self.state.halt(reason);
                    break;
                }
            };

            // Decode instruction
            let instr = match decode(word) {
                Ok(i) => i,
                Err(_) => {
                    self.state.halt(HaltReason::InvalidInstruction {
                        pc: self.state.pc,
                        word,
                    });
                    break;
                }
            };

            // Execute instruction
            if let Err(e) = execute(&instr, &mut self.state, &mut self.io) {
                match e {
                    RuntimeError::Halt(reason) => {
                        self.state.halt(reason);
                        break;
                    }
                }
            }

            // Increment cycle
            self.state.cycle += 1;

            // Optional trace logging
            if self.config.trace_enabled {
                tracing::trace!(
                    "cycle={} pc={:08X} instr={:08X}",
                    self.state.cycle,
                    self.state.pc,
                    word
                );
            }
        }

        Ok(ExecutionResult {
            outputs: self.io.take_outputs(),
            commitments: self.io.take_commitments(),
            cycles: self.state.cycle,
            halt_reason: self.state.halt_reason.clone().unwrap_or(HaltReason::Halt),
        })
    }
}
