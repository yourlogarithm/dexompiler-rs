#[macro_use]
extern crate lazy_static;

mod dex;
mod errors;
mod manifest;

use ::dex::DexReader;
use bitcode::{Decode, Encode};
use dex::{get_methods, Method};
use log::{error, warn};
use regex::{bytes::Regex as BytesRegex, Regex};
use serde::Serialize;
use std::io::{Read, Seek};
use zip::ZipArchive;

pub use errors::ApkParseError;

lazy_static! {
    static ref DEX_MAGIC: BytesRegex =
        BytesRegex::new(r"\x64\x65\x78\x0A\x30\x33[\x35-\x39]\x00").unwrap();
}

/// Represents an APK (Android Package) with metadata and methods.
#[derive(Debug, Serialize, Encode, Decode)]
pub struct Apk {
    #[serde(rename = "man")]
    pub manifest: Option<manifest::Manifest>, // Optional manifest information

    // Topologically sorted methods within the APK
    #[serde(rename = "mth")]
    pub methods: Vec<Method>,
}

impl Apk {
    /// Converts the APK to a compact representation with reduced method information.
    ///
    /// ### Returns
    /// A `CompactApk` with the same manifest and compacted method data.
    pub fn to_compact(self) -> CompactApk {
        self.into()
    }
}

/// A compact representation of a method with opcode data.
///
/// This is useful for reducing the APK size while maintaining method information.
pub type CompactMethod = Vec<u8>;

/// A compact version of the `Apk` struct where methods are stored as a vector of opcodes.
///
/// This reduces the APK's overall size and can be used for efficient storage or transfer.
#[derive(Debug, Serialize, Encode, Decode)]
pub struct CompactApk {
    /// Optional manifest information
    #[serde(rename = "man")]
    pub manifest: Option<manifest::Manifest>,
    /// Compact method representations as opcode vectors.
    #[serde(rename = "mth")]
    pub methods: Vec<CompactMethod>,
}

/// Converts an `Apk` into a `CompactApk` by compacting method information.
///
/// This transformation reduces the overall size of the APK.
impl From<Apk> for CompactApk {
    fn from(apk: Apk) -> Self {
        CompactApk {
            manifest: apk.manifest,
            methods: apk
                .methods
                .into_iter()
                .map(|method| method.insns.into_iter().map(|i| i.opcode as u8).collect())
                .collect(),
        }
    }
}

/// Parses a source of bytes (e.g., a .apk archive) into an `Apk` structure.
///
/// This function reads an APK archive, extracting its manifest and DEX (Dalvik Executable) files,
/// and constructs an `Apk` with a sorted list of methods.
///
/// ### Arguments
/// * `apk`: A reader and seeker that represents the APK source.
///
/// ### Returns
/// * `Result<Apk, ApkParseError>`: A successful parse yields an `Apk`, while failure results in an `ApkParseError`.
///
/// ### Example
/// ```rust
/// use dexompiler::parse;
///
/// fn main() {
///     let file = std::fs::File::open("tests/example.apk").unwrap();
///     let apk = parse(file).unwrap();
///     println!("{apk:?}");
///     let compact = apk.to_compact();
///     println!("{compact:?}");
/// }
/// ```
pub fn parse<R: Read + Seek>(apk: R) -> Result<Apk, ApkParseError> {
    let mut zip_archive = ZipArchive::new(apk)?;
    let mut manifest = None;
    let mut dexes = Vec::new();

    for i in 0..zip_archive.len() {
        let mut file = match zip_archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                error!("Error reading file at index {i}: {e}");
                continue;
            }
        };

        let mut buf = Vec::new();
        if let Err(e) = file.read_to_end(&mut buf) {
            warn!("Error reading file: {e}");
            continue;
        }

        if file.name() == "AndroidManifest.xml" {
            if manifest.is_some() {
                warn!("Multiple AndroidManifest.xml files found in APK");
            } else {
                manifest = manifest::parse(&buf)?;
            }
        } else {
            if DEX_MAGIC.is_match(&buf) {
                match DexReader::from_vec(buf) {
                    Ok(dex) => dexes.push(dex),
                    Err(e) => error!("{e}"),
                }
            }
        }
    }

    let regexes = if let Some(ref m) = manifest {
        match m
            .activities
            .iter()
            .chain(m.services.iter())
            .chain(m.receivers.iter())
            .chain(m.providers.iter())
            .map(|s| Regex::new(s.replace('.', "/").as_str()))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(regexes) => Some(regexes),
            Err(e) => {
                error!("{e}");
                None
            }
        }
    } else {
        None
    };

    Ok(Apk {
        manifest,
        methods: get_methods(&dexes, regexes)?,
    })
}
