use super::instruction::InstructionError;
use thiserror::Error;
use zip::result::ZipError;

#[derive(Debug, Error)]
pub enum ApkParseError {
    #[error("Failed to read archive: {0}")]
    ZipError(ZipError),
    #[error("Failed to parse DEX file: {0}")]
    DexError(DexError),
}

impl From<ZipError> for ApkParseError {
    fn from(e: ZipError) -> Self {
        ApkParseError::ZipError(e)
    }
}

impl From<DexError> for ApkParseError {
    fn from(e: DexError) -> Self {
        ApkParseError::DexError(e)
    }
}

#[derive(Debug, Error)]
pub struct DexError {
    pub class_name: String,
    pub method_name: String,
    pub source: InstructionError,
}

impl std::fmt::Display for DexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DexError class_id - {}, method_id - {}: {}",
            self.class_name, self.method_name, self.source
        )
    }
}
