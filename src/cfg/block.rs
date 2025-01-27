use std::collections::HashMap;

use crate::instruction::{Instruction, Opcode};

use super::{CallGraphError, MethodControlFlowGraph};

#[derive(Debug)]
pub(crate) struct BasicBlock {
    pub start: u32,
    pub end: u32,
    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    pub fn from_method_cfg(
        MethodControlFlowGraph {
            mut instructions,
            mut switch_branch_targets,
            leaders,
            ..
        }: MethodControlFlowGraph,
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
                            entry.push(MethodControlFlowGraph::target(*op, *addr, &format)?);
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
                    other => log::warn!("Unexpected instruction: {other:?}"),
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
                    .collect(),
            });
        }
        Ok(blocks)
    }
}
