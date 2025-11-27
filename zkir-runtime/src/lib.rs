//! ZK IR Runtime
//!
//! Execute ZK IR programs and generate traces for proving.

pub mod vm;
pub mod state;
pub mod memory;
pub mod io;
pub mod decode;
pub mod error;

pub use vm::{VM, VMConfig, ExecutionResult};
pub use state::{VMState, HaltReason};
pub use error::RuntimeError;

/// Simple execution
pub fn run(program: zkir_spec::Program, inputs: Vec<u32>) -> Result<Vec<u32>, RuntimeError> {
    let vm = VM::new(program, inputs, VMConfig::default());
    Ok(vm.run()?.outputs)
}
