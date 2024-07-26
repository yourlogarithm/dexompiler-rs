mod error;
pub use error::CallGraphError;

use std::collections::{HashMap, HashSet};

use log::warn;

use crate::instruction::{Format, Instruction, InstructionError, Opcode};

#[derive(Debug)]
pub struct BasicBlock {
    pub start: u32,
    pub end: u32,
    pub instructions: Vec<Instruction>,
    pub successors: Vec<usize>,
    pub predecessors: Vec<usize>,
    pub dominators: HashSet<usize>,
}

#[derive(Debug)]
pub struct Loop {
    pub header: usize,
    pub blocks: HashSet<usize>,
}

pub type LoopSet = Vec<Loop>;

#[derive(Debug, Default)]
pub struct MethodCFG {
    instructions: Vec<(u32, Instruction)>,
    switch_payload_origin: HashMap<u32, Vec<u32>>,
    switch_branch_targets: HashMap<u32, Vec<u32>>,
    leaders: HashSet<u32>,
}

impl MethodCFG {
    pub fn new() -> Self {
        Self::default()
    }

    fn target(op: Opcode, address: u32, format: &Format) -> Result<u32, CallGraphError> {
        let res = address as i32 + format.offset().ok_or(InstructionError::BadFormat(op))?;
        Ok(res as u32)
    }

    pub fn add_instruction(
        &mut self,
        address: u32,
        instruction: Instruction,
    ) -> Result<(), CallGraphError> {
        match instruction {
            Instruction::Regular { op, ref format } => {
                match op {
                    Opcode::Goto | Opcode::Goto16 | Opcode::Goto32 => {
                        self.leaders.insert(Self::target(op, address, format)?);
                    }
                    Opcode::PackedSwitch | Opcode::SparseSwitch => {
                        self.leaders.insert(address + format.len() as u32);
                        self.switch_payload_origin
                            .entry(Self::target(op, address, format)?)
                            .or_default()
                            .push(address);
                    }
                    Opcode::FillArrayData => {}
                    _ => {
                        if let Some(offset) = format.offset() {
                            self.leaders.insert((address as i32 + offset) as u32);
                            self.leaders.insert(address + format.len() as u32);
                        }
                    }
                }
                self.instructions.push((address, instruction));
                if self.instructions.len() == 1 {
                    self.leaders.insert(address);
                }
            }
            Instruction::SwitchPayload { ref kv, .. } => {
                let call_position = self
                    .switch_payload_origin
                    .remove(&address)
                    .ok_or(CallGraphError::MissingSwitchOrigin(address))?;
                for v in kv.values().cloned() {
                    for origin in call_position.iter().cloned() {
                        let target = (origin as i32 + v) as u32;
                        self.leaders.insert(target);
                        self.switch_branch_targets
                            .entry(origin)
                            .or_default()
                            .push(target);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn into_basic_blocks(self) -> Result<Vec<BasicBlock>, CallGraphError> {
        let mut instructions = self.instructions;
        let Some(&max) = instructions.iter().map(|(addr, _)| addr).max() else {
            return Ok(vec![]);
        };
        let mut blocks = Vec::new();
        let mut switch_branch_targets = self.switch_branch_targets;
        let mut sorted_leaders: Vec<_> = self.leaders.iter().cloned().collect();
        sorted_leaders.sort_unstable();
        let mut predecessors_mapping: HashMap<u32, Vec<usize>> = HashMap::new();
        let mut successors_mapping: HashMap<usize, Vec<u32>> = HashMap::new();
        for (i, &start) in sorted_leaders.iter().enumerate() {
            let end = if i < sorted_leaders.len() - 1 {
                sorted_leaders[i + 1] - 1
            } else {
                max
            };
            let idx = blocks.len();
            let local_instructions: Vec<_> = instructions
                .extract_if(|(pos, _)| *pos >= start && *pos <= end)
                .collect();
            let entry = successors_mapping.entry(idx).or_default();
            for (addr, inst) in local_instructions.iter() {
                match inst {
                    Instruction::Regular { op, format } => match op {
                        Opcode::Goto | Opcode::Goto16 | Opcode::Goto32 => {
                            entry.push(Self::target(*op, *addr, &format)?);
                        }
                        Opcode::PackedSwitch | Opcode::SparseSwitch => {
                            entry.extend(
                                switch_branch_targets
                                    .remove(&addr)
                                    .ok_or(CallGraphError::MissingSwitchOrigin(*addr))?,
                            );
                            entry.push(addr + format.len() as u32);
                        }
                        Opcode::FillArrayData => {}
                        _ => {
                            if let Some(offset) = format.offset() {
                                entry.push((*addr as i32 + offset) as u32);
                                entry.push(*addr + format.len() as u32);
                            } else {
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
            blocks.push((
                start,
                end,
                local_instructions
                    .into_iter()
                    .map(|(_, inst)| inst)
                    .collect::<Vec<_>>(),
            ));
        }

        let mut start_to_idx: HashMap<_, _> = blocks
            .iter()
            .enumerate()
            .map(|(i, (start, _, _))| (*start, i))
            .collect();

        let blocks = blocks
            .into_iter()
            .enumerate()
            .map(|(i, (start, end, instructions))| {
                let predecessors = predecessors_mapping.remove(&start).unwrap_or_default();
                let successors = successors_mapping
                    .remove(&i)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|addr| start_to_idx.remove(&addr))
                    .collect();
                BasicBlock {
                    start,
                    end,
                    instructions,
                    predecessors,
                    successors,
                    dominators: HashSet::new()
                }
            })
            .collect();

        Ok(blocks)
    }
}
