#[macro_use]
extern crate lazy_static;

use std::{
    collections::{HashMap, HashSet},
    hash::RandomState,
    io::{Read, Seek},
};

use dex::{code::CodeItem, method::Method, Dex, DexReader};
use errors::ApkParseError;

use instruction::{Instruction, InstructionError, Opcode};
use log::{error, warn};
use regex::bytes::Regex;
use zip::ZipArchive;

mod block;
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
                let Some(code_item) = method.code() else {
                    continue;
                };
                parse_method(code_item, method, &class_name)?;
            }
        }
    }
    Ok(())
}

fn parse_method(
    code_item: &CodeItem,
    method: &Method,
    class_name: &String,
) -> Result<(), InstructionError> {
    let mut iter = code_item.insns().iter().cloned().peekable();
    let mut pos = 0;
    let mut instructions = Vec::new();

    let mut switch_position_mapping = HashMap::new();
    // FIXME: Wrong, any basick block between the try and catch will be considered predecessor of the catch
    // FIXME: This considers only the first basick block start address
    let mut entry_points: HashMap<_, HashSet<_, RandomState>> = code_item
        .tries()
        .iter()
        .flat_map(|t| {
            t.catch_handlers().into_iter().map(|h| {
                (
                    h.addr(),
                    HashSet::from_iter(std::iter::once(t.start_addr())),
                )
            })
        })
        .collect();

    while let Some(inst) = Instruction::try_from_code(&mut iter)? {
        let size = match inst {
            Instruction::Regular { op, ref format } => {
                if op == Opcode::PackedSwitch || op == Opcode::SparseSwitch {
                    let offset = format.offset().ok_or(InstructionError::BadFormat(op))?;
                    switch_position_mapping.insert(pos as i32 + offset, pos);
                } else if let Some(offset) = format.offset() {
                    let entry = entry_points
                        .entry((pos as i32 + offset) as u64)
                        .or_default();
                    entry.insert(pos);
                }
                format.len() as u32
            }
            Instruction::Switch { ref kv, code_units } => {
                let call_position = switch_position_mapping
                    .remove(&(pos as i32))
                    .ok_or(InstructionError::MissingSwitchOrigin(pos))?;
                let entry = entry_points.entry(call_position as u64).or_default();
                entry.extend(kv.values().map(|&v| (call_position as i32 + v) as u32));
                code_units as u32
            }
            Instruction::FillArrayData {
                size,
                element_width,
                ..
            } => (size as u32 * element_width as u32 + 1) / 2 + 4,
        };
        instructions.push((pos, inst));
        pos += size;
    }

    println!("{class_name}.{} <=> {entry_points:?}", method.name());

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
