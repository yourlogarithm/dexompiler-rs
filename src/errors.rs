use thiserror::Error;

use crate::instruction::Opcode;

#[derive(Debug, Error)]
pub enum InstructionError {
    #[error("Instruction is too short for {0:?}")]
    TooShort(u8),
    #[error("Opcode {0} does not exist")]
    BadOpcode(u8),
    #[error("Opcode {0:?} has bad format")]
    BadFormat(Opcode),
    #[error("Code ended abruptly")]
    End,
}

#[derive(Debug, Error)]
pub enum CallGraphError {
    #[error("Missing switch origin for {0}")]
    MissingSwitchOrigin(u32),
    #[error("Instruction error - {0}")]
    InstructionError(#[from] InstructionError),
}

#[derive(Debug, Error)]
pub enum DexError {

}

