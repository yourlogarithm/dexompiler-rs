#[macro_use]
extern crate lazy_static;

use std::io::{Read, Seek};

use dex::{Dex, DexReader};
use errors::ApkParseError;

use instruction::{Instruction, InstructionError};
use log::{error, warn};
use regex::bytes::Regex;
use zip::ZipArchive;

mod errors;
mod instruction;

lazy_static! {
    static ref DEX_MAGIC: Regex = Regex::new(r"\x64\x65\x78\x0A\x30\x33[\x35-\x39]\x00").unwrap();
}

pub fn parse<R: Read + Seek>(apk: R) -> Result<(), ApkParseError> {
    let mut zip_archive = ZipArchive::new(apk)?;
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

        if DEX_MAGIC.is_match(&buf) {
            match DexReader::from_vec(buf) {
                Ok(dex) => dexes.push(dex),
                Err(e) => error!("{e}"),
            }
        }
    }

    parse_dexes(dexes).unwrap();

    Ok(())
}

fn parse_dexes(dexes: Vec<Dex<Vec<u8>>>) -> Result<(), InstructionError> {
    for dex in dexes.into_iter() {
        for class in dex.classes().filter_map(Result::ok) {
            let class_name = class.jtype().to_java_type();
            for method in class.methods() {
                let method_name = method.name();
                println!("{class_name}.{method_name}");
                if let Some(code_item) = method.code() {
                    let mut iter = code_item.insns().iter().cloned().peekable();
                    while let Some(inst) = Instruction::try_from_code(&mut iter)? {
                        println!("{:?}", inst);
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let fdroid = std::fs::File::open("F-Droid.apk").unwrap();
        super::parse(fdroid).unwrap();
    }
}
