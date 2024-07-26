#![feature(extract_if)]

#[macro_use]
extern crate lazy_static;

use std::{
    collections::{HashSet, VecDeque},
    io::{Read, Seek},
};

use cfg::{BasicBlock, CallGraphError, Loop, LoopSet};
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
    let mut blocks = cfg.into_basic_blocks()?;
    compute_dominators(&mut blocks);
    let loops = compute_natural_loops(&blocks);
    // println!("Loops:");
    // for l in loops {
    //     println!("{l:?}");
    // }
    println!("{blocks:#?}");
    Ok(())
}

pub fn compute_dominators(blocks: &mut Vec<BasicBlock>) {
    let n_blocks = blocks.len();

    for block in blocks.iter_mut() {
        block.dominators = (0..n_blocks).collect();
    }

    if !blocks.is_empty() {
        blocks[0].dominators = HashSet::from([0]);
    }

    let mut changed = true;
    while changed {
        changed = false;

        for i in 1..n_blocks {
            let mut new_doms: HashSet<usize> = (0..n_blocks).collect();

            for &pred in &blocks[i].predecessors {
                new_doms = new_doms
                    .intersection(&blocks[pred].dominators)
                    .cloned()
                    .collect();
            }

            new_doms.insert(i);

            if new_doms != blocks[i].dominators {
                blocks[i].dominators = new_doms;
                changed = true;
            }
        }
    }
}

pub fn natural_loop_for_edge(blocks: &[BasicBlock], header: usize, tail: usize) -> Loop {
    let mut loop_struct = Loop {
        header,
        blocks: HashSet::from([header]),
    };

    let mut work_list = VecDeque::new();

    if header != tail {
        loop_struct.blocks.insert(tail);
        work_list.push_back(tail);
    }

    while let Some(block) = work_list.pop_front() {
        for &pred in &blocks[block].predecessors {
            if !loop_struct.blocks.contains(&pred) {
                loop_struct.blocks.insert(pred);
                work_list.push_back(pred);
            }
        }
    }

    loop_struct
}

pub fn compute_natural_loops(blocks: &[BasicBlock]) -> LoopSet {
    let mut loop_set = LoopSet::new();

    for (i, block) in blocks.iter().enumerate().skip(1) {
        for &succ in &block.successors {
            if block.dominators.contains(&succ) {
                loop_set.push(natural_loop_for_edge(blocks, succ, i));
            }
        }
    }

    loop_set
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
