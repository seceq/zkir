//! Runtime errors

use thiserror::Error;
use crate::state::HaltReason;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("VM halted: {0:?}")]
    Halt(HaltReason),
}

impl From<HaltReason> for RuntimeError {
    fn from(reason: HaltReason) -> Self {
        RuntimeError::Halt(reason)
    }
}
