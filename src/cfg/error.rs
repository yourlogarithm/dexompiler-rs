use thiserror::Error;

use crate::instruction::InstructionError;


#[derive(Debug, Error)]
pub enum CallGraphError {
    #[error("Missing switch origin for {0}")]
    MissingSwitchOrigin(u32),
    #[error("Instruction error - {0}")]
    InstructionError(#[from] InstructionError),
}