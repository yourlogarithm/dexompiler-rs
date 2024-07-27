#![feature(extract_if)]

#[macro_use]
extern crate lazy_static;

use std::io::{Read, Seek};

use cfg::{BasicBlock, CallGraphError};
use dex::{Dex, DexReader};
use errors::ApkParseError;

use instruction::Instruction;
use log::{error, warn};
use regex::bytes::Regex;
use zip::ZipArchive;

mod cfg;
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

fn parse_dexes(dexes: Vec<Dex<Vec<u8>>>) -> Result<(), CallGraphError> {
    for dex in dexes.into_iter() {
        for class in dex.classes().filter_map(Result::ok) {
            for method in class.methods() {
                let Some(code_item) = method.code() else {
                    continue;
                };
                // TODO: No clone
                parse_method(code_item.insns().to_vec())?;
            }
        }
    }
    Ok(())
}

fn parse_method(code: Vec<u16>) -> Result<(), CallGraphError> {
    let mut pos = 0;
    let mut iter = code.into_iter().peekable();
    let mut cfg = cfg::MethodCFG::new();
    while let Some(inst) = Instruction::try_from_code(&mut iter)? {
        let size = match inst {
            Instruction::Regular { ref format, .. } => format.len() as u32,
            Instruction::SwitchPayload { code_units, .. } => code_units as u32,
            Instruction::FillArrayDataPayload { code_units, .. } => code_units,
        };
        cfg.add_instruction(pos, inst)?;
        pos += size;
    }
    let mut blocks = BasicBlock::from_method_cfg(cfg)?;
    println!("{blocks:#?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::parse_method;

    // #[test]
    // fn it_works() {
    //     let fdroid = std::fs::File::open("F-Droid.apk").unwrap();
    //     super::parse(fdroid).unwrap();
    //     // panic!()
    // }

    #[test]
    fn it_works2() {
        let bytes = std::fs::read("method.bin").unwrap();
        let words = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()));
        parse_method(words.collect()).unwrap();
        panic!()
    }
}
