use zip::result::ZipError;
use thiserror::Error;
use axmldecoder::ParseError;

use crate::dex::DexError;

#[derive(Debug, Error)]
pub enum ApkParseError {
    #[error("Failed to read archive: {0}")]
    ZipError(ZipError),
    #[error("Failed to parse manifest: {0}")]
    ManifestError(ParseError),
    #[error("Failed to parse DEX file: {0}")]
    DexError(DexError)
}

impl From<ZipError> for ApkParseError {
    fn from(e: ZipError) -> Self {
        ApkParseError::ZipError(e)
    }
}

impl From<ParseError> for ApkParseError {
    fn from(e: ParseError) -> Self {
        ApkParseError::ManifestError(e)
    }
}

impl From<DexError> for ApkParseError {
    fn from(e: DexError) -> Self {
        ApkParseError::DexError(e)
    }
}