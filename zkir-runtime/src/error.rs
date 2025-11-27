//! Runtime errors

use thiserror::Error;
use crate::state::HaltReason;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("VM halted: {0:?}")]
    Halt(HaltReason),
}
