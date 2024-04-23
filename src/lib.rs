#[macro_use]
extern crate lazy_static;

mod dex;
mod errors;
mod manifest;

use bitcode::{Decode, Encode};
use ::dex::DexReader;
use dex::{get_methods, Method};
use log::{error, warn};
use regex::{Regex, bytes::Regex as BytesRegex};
use serde::Serialize;
use std::io::{Read, Seek};
use zip::ZipArchive;

pub use errors::ApkParseError;

lazy_static! {
    static ref DEX_MAGIC: BytesRegex = BytesRegex::new(r"\x64\x65\x78\x0A\x30\x33[\x35-\x39]\x00").unwrap();
}

#[derive(Debug, Serialize, Encode, Decode)]
pub struct Apk {
    #[serde(rename = "man")]
    pub manifest: Option<manifest::Manifest>,
    // Topologically sorted methods
    #[serde(rename = "mth")]
    pub methods: Vec<Method>,
}

#[derive(Debug, Serialize, Encode, Decode)]
pub struct CompactMethod(Vec<u8>);

impl From<Method> for CompactMethod {
    fn from(method: Method) -> Self {
        CompactMethod(method.insns.into_iter().map(|i| i.opcode as u8).collect())
    }
}

#[derive(Debug, Serialize, Encode, Decode)]
pub struct CompactApk {
    #[serde(rename = "man")]
    pub manifest: Option<manifest::Manifest>,
    #[serde(rename = "mth")]
    pub methods: Vec<CompactMethod>,
}

impl From<Apk> for CompactApk {
    fn from(apk: Apk) -> Self {
        CompactApk {
            manifest: apk.manifest,
            methods: apk.methods.into_iter().map(CompactMethod::from).collect(),
        }
    }
}

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
