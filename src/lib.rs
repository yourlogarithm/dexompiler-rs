#[macro_use]
extern crate lazy_static;

mod dex;
mod errors;
mod permissions;

use log::{debug, error, warn};
use std::io::{Read, Seek};
use regex::bytes::Regex;
use zip::ZipArchive;

pub use errors::ApkParseError;

lazy_static! {
    static ref DEX_MAGIC: Regex = Regex::new(r"\x64\x65\x78\x0A\x30\x33[\x35-\x39]\x00").unwrap();
}

pub fn parse<R: Read + Seek>(apk: R) -> Result<(), ApkParseError> {
    let mut zip_archive = ZipArchive::new(apk)?;
    let mut buf = Vec::new();
    let mut permissions = None;
    let mut dexes = Vec::new();
    for i in 0..zip_archive.len() {
        let mut file = match zip_archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                error!("Error reading file at index {i}: {e}");
                continue;
            }
        };
        if let Err(e) = file.read_to_end(&mut buf) {
            debug!("Error reading file: {e}");
            continue;
        }
        if file.name() == "AndroidManifest.xml" {
            if permissions.is_some() {
                warn!("Multiple AndroidManifest.xml files found in APK");
            } else {
                permissions = permissions::parse(&buf)?;
            }
        } else {
            if DEX_MAGIC.is_match(&buf) {
                dexes.push(dex::parse(&buf)?);
            }
        }
    }
    println!("{permissions:?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse;
    use std::fs::File;

    #[test]
    fn it_works() {
        let file = File::open("F-Droid.apk").unwrap();
        parse(file).unwrap();
    }
}
