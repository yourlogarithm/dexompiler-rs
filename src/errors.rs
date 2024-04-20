use zip::result::ZipError;
use thiserror::Error;
use axmldecoder::ParseError;

#[derive(Debug, Error)]
pub enum ApkParseError {
    #[error("Failed to read archive: {0}")]
    ZipError(ZipError),
    #[error("Failed to parse manifest")]
    ManifestError(ParseError),
    #[error("Failed to parse DEX file")]
    DexError()
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