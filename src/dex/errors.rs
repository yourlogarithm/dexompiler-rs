use std::num::NonZeroUsize;

use thiserror::Error;

use super::Opcode;

#[derive(Debug, Error)]
pub struct DexError {
    pub class_name: String,
    pub method_name: String,
    pub source: InstructionError
}

impl std::fmt::Display for DexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DexError class_id - {}, method_id - {}: {}", self.class_name, self.method_name, self.source)
    }
}

#[derive(Debug, Error)]
pub enum InstructionError {
    #[error("Instruction is too short for {opcode:?}, expected {expected}, found {actual}")]
    TooShort {
        offset: usize,
        opcode: Opcode,
        expected: NonZeroUsize,
        actual: NonZeroUsize
    },
    #[error("Opcode {1} at index {0} does not exist")]
    BadOpcode(usize, u8),
    #[error("Code ended")]
    End
}