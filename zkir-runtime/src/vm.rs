//! Virtual Machine (placeholder)

use zkir_spec::Program;
use crate::state::{VMState, HaltReason};
use crate::io::IOHandler;
use crate::error::RuntimeError;

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
    _config: VMConfig,
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
            _config: config,
        }
    }

    pub fn run(mut self) -> Result<ExecutionResult, RuntimeError> {
        // Placeholder: just halt immediately
        self.state.halt(HaltReason::Halt);

        Ok(ExecutionResult {
            outputs: self.io.take_outputs(),
            commitments: self.io.take_commitments(),
            cycles: self.state.cycle,
            halt_reason: self.state.halt_reason.clone().unwrap_or(HaltReason::Halt),
        })
    }
}
