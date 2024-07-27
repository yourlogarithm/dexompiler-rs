use bitvec::prelude::*;
use log::warn;
use std::collections::{HashMap, HashSet};

use crate::instruction::{Instruction, Opcode};

use super::{r#loop::Loop, CallGraphError, MethodCFG};

#[derive(Debug)]
pub struct BasicBlock {
    pub start: u32,
    pub end: u32,
    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    pub fn from_method_cfg(
        MethodCFG {
            mut instructions,
            mut switch_branch_targets,
            leaders,
            ..
        }: MethodCFG,
    ) -> Result<Vec<BasicBlock>, CallGraphError> {
        let Some(&max) = instructions.iter().map(|(addr, _)| addr).max() else {
            return Ok(vec![]);
        };
        let mut blocks = Vec::new();
        let mut sorted_leaders: Vec<_> = leaders.into_iter().collect();
        sorted_leaders.sort_unstable();
        let mut predecessors_mapping: HashMap<u32, Vec<usize>> = HashMap::new();
        let mut successors_mapping: HashMap<usize, Vec<u32>> = HashMap::new();
        let len = sorted_leaders.len();
        for (i, &start) in sorted_leaders.iter().enumerate() {
            let end = if i < len - 1 {
                sorted_leaders[i + 1] - 1
            } else {
                max
            };
            let idx = blocks.len();
            let local_instructions: Vec<_> = instructions
                .extract_if(|(pos, _)| *pos >= start && *pos <= end)
                .collect();
            let entry = successors_mapping.entry(idx).or_default();
            for (i, (addr, inst)) in local_instructions.iter().enumerate() {
                match inst {
                    Instruction::Regular { op, format } => match op {
                        Opcode::Goto | Opcode::Goto16 | Opcode::Goto32 => {
                            entry.push(MethodCFG::target(*op, *addr, &format)?);
                        }
                        Opcode::PackedSwitch | Opcode::SparseSwitch => {
                            entry.extend(
                                switch_branch_targets
                                    .remove(&addr)
                                    .ok_or(CallGraphError::MissingSwitchOrigin(*addr))?,
                            );
                            entry.push(addr + format.len() as u32);
                        }
                        Opcode::FillArrayData
                        | Opcode::ReturnVoid
                        | Opcode::Return
                        | Opcode::ReturnWide
                        | Opcode::ReturnObject
                        | Opcode::Throw => {}
                        _ => {
                            if let Some(offset) = format.offset() {
                                entry.push((*addr as i32 + offset) as u32);
                                entry.push(*addr + format.len() as u32);
                            } else if i == local_instructions.len() - 1 {
                                entry.push(*addr + format.len() as u32);
                            }
                        }
                    },
                    other => warn!("Unexpected instruction: {other:?}"),
                }
            }
            for successor in successors_mapping[&idx].iter().cloned() {
                predecessors_mapping.entry(successor).or_default().push(idx);
            }
            blocks.push(BasicBlock {
                start,
                end,
                instructions: local_instructions
                    .into_iter()
                    .map(|(_, inst)| inst)
                    .collect::<Vec<_>>(),
            });
        }
        let start_to_idx: HashMap<_, _> = blocks
            .iter()
            .enumerate()
            .map(|(i, block)| (block.start, i))
            .collect();
        let mut dominators = vec![bitvec![1; blocks.len()]; blocks.len()];
        dominators[0] = bitvec![0; blocks.len()];
        dominators[0].set(0, true);
        loop {
            let mut changed = false;
            for (i, block) in blocks.iter().enumerate().skip(1) {
                let Some(predecessors) = predecessors_mapping.get(&block.start) else {
                    log::warn!("No predecessors for block at {:x}", block.start);
                    continue;
                };
                for pred in predecessors {
                    let old = dominators[i].clone();
                    let pred_dominators = dominators[*pred].clone();
                    dominators[i] = old.clone() & pred_dominators;
                    dominators[i].set(i, true);
                    if dominators[i] != old {
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        let mut loop_set = vec![];
        for (i, block) in blocks.iter().enumerate().skip(1) {
            let Some(successors) = successors_mapping.get(&i) else {
                log::warn!("No successors for block at {:x}", block.start);
                continue;
            };
            for successor in successors.into_iter().map(|addr| start_to_idx[&addr]) {
                if dominators[i][successor] {
                    loop_set.push(Self::natural_for_loop_edge(
                        &blocks[successor],
                        block,
                        &blocks,
                        &predecessors_mapping,
                    ))
                }
            }
        }
        println!("Loop set {loop_set:#?}",);
        todo!()
    }

    fn natural_for_loop_edge<'a>(
        header: &'a BasicBlock,
        tail: &'a BasicBlock,
        blocks: &'a [BasicBlock],
        predecessors_mapping: &HashMap<u32, Vec<usize>>,
    ) -> Loop<'a> {
        let mut worklist = Vec::new();
        let mut loop_ = Loop::new(header);
        if header.start != tail.start {
            loop_.add_block(tail);
            worklist.push(tail);
        }
        while let Some(block) = worklist.pop() {
            let Some(predecessors) = predecessors_mapping.get(&block.start) else {
                log::warn!("No predecessors for block at {:x}", block.start);
                continue;
            };
            for predecessor in predecessors.into_iter().map(|&idx| &blocks[idx]) {
                if !loop_.contains(predecessor) {
                    loop_.add_block(predecessor);
                    worklist.push(predecessor);
                }
            }
        }
        return loop_;
    }

    // pub fn into_basic_blocks(self) -> Result<Vec<BasicBlock>, CallGraphError> {
    //     let blocks = blocks
    //         .into_iter()
    //         .enumerate()
    //         .map(|(i, (start, end, instructions))| {
    //             let predecessors = predecessors_mapping.remove(&start).unwrap_or_default();
    //             let successors = successors_mapping
    //                 .remove(&i)
    //                 .unwrap_or_default()
    //                 .into_iter()
    //                 .filter_map(|addr| start_to_idx.remove(&addr))
    //                 .collect();
    //             BasicBlock {
    //                 start,
    //                 end,
    //                 instructions,
    //                 predecessors,
    //                 successors,
    //                 dominators: HashSet::new(),
    //             }
    //         })
    //         .collect();

    //     Ok(blocks)
    // }
}
