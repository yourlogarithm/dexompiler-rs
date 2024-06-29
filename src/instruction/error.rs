use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstructionError {
    #[error("Instruction is too short for {0:?}")]
    TooShort(u8),
    #[error("Opcode {0} does not exist")]
    BadOpcode(u8),
    #[error("Code ended abruptly")]
    End,
}
