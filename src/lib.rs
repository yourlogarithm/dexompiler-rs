#![feature(extract_if)]

use cfg::BasicBlock;
use dex::{class::Class, code::CodeItem, method::Method, DexReader, Error};

use errors::CallGraphError;
use instruction::Instruction;

mod cfg;
mod errors;
mod instruction;

pub fn parse(buf: impl AsRef<[u8]>) -> Result<(), Error> {
    let dex = DexReader::from_vec(buf)?;
    for result_class in dex.classes() {
        let Class { direct_methods, virtual_methods, .. } = match result_class {
            Ok(class) => class,
            Err(e) => {
                log::error!("{e}");
                continue;
            }
        };
        for Method { code, name, .. } in direct_methods.into_iter().chain(virtual_methods) {
            log::trace!("{name}");
            let Some(CodeItem { insns, .. }) = code else {
                log::debug!("{name} - missing code");
                continue;
            };
            if let Err(e) = parse_method(insns) {
                log::error!("{e}");
            }
        }
    }
    Ok(())
}

fn parse_method(insns: Vec<u16>) -> Result<(), CallGraphError> {
    let mut pos = 0;
    let mut iter = insns.into_iter().peekable();
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
    let blocks = BasicBlock::from_method_cfg(cfg)?;
    println!("{blocks:#?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use zip::ZipArchive;

    use crate::{parse, parse_method};

    const DEX_MAGICS: [&[u8; 8]; 5] = [
        b"\x64\x65\x78\x0A\x30\x33\x39\x00",
        b"\x64\x65\x78\x0A\x30\x33\x38\x00",
        b"\x64\x65\x78\x0A\x30\x33\x37\x00",
        b"\x64\x65\x78\x0A\x30\x33\x36\x00",
        b"\x64\x65\x78\x0A\x30\x33\x35\x00",
    ];

    #[test]
    fn it_works() {
        let apk = std::fs::read("F-Droid.apk").unwrap();
        let cursor = std::io::Cursor::new(apk);
        let mut zip_archive = ZipArchive::new(cursor).unwrap();
        for i in 0..zip_archive.len() {
            let mut file = match zip_archive.by_index(i) {
                Ok(file) => file,
                Err(e) => {
                    log::error!("Error reading file at index {i}: {e}");
                    continue;
                }
            };

            let mut buf = Vec::new();
            if let Err(e) = file.read_to_end(&mut buf) {
                log::warn!("Error reading file: {e}");
                continue;
            }

            for magic in DEX_MAGICS {
                if magic == &buf[..8] {
                    if let Err(e) = parse(buf) {
                        log::error!("{e}");
                    }
                    break;
                }
            }
        }
    }

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
