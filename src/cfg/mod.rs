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
    pub successors: Vec<u32>,
}

#[derive(Debug, Default)]
pub struct ControlFlowGraph {
    instructions: Vec<(u32, Instruction)>,
    switch_payload_origin: HashMap<u32, Vec<u32>>,
    switch_branch_targets: HashMap<u32, Vec<u32>>,
    leaders: HashSet<u32>,
}

impl ControlFlowGraph {
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
        let mut blocks = Vec::new();
        let mut instructions = self.instructions;
        let mut switch_branch_targets = self.switch_branch_targets;
        let Some(&max) = instructions.iter().map(|(addr, _)| addr).max() else {
            return Ok(blocks);
        };
        let mut sorted_leaders: Vec<_> = self.leaders.iter().cloned().collect();
        sorted_leaders.sort_unstable();

        for (i, &start) in sorted_leaders.iter().enumerate() {
            let end = if i < sorted_leaders.len() - 1 {
                sorted_leaders[i + 1] - 1
            } else {
                max
            };
            let instructions: Vec<_> = instructions
                .extract_if(|(pos, _)| *pos >= start && *pos <= end)
                .collect();
            let mut successors = Vec::new();
            for (addr, inst) in instructions.iter() {
                match inst {
                    Instruction::Regular { op, format } => match op {
                        Opcode::Goto | Opcode::Goto16 | Opcode::Goto32 => {
                            successors.push(Self::target(*op, *addr, &format)?);
                        }
                        Opcode::PackedSwitch | Opcode::SparseSwitch => {
                            successors.extend(
                                switch_branch_targets
                                    .remove(&addr)
                                    .ok_or(CallGraphError::MissingSwitchOrigin(*addr))?,
                            );
                            successors.push(addr + format.len() as u32);
                        }
                        Opcode::FillArrayData => {}
                        _ => {
                            if let Some(offset) = format.offset() {
                                successors.push((*addr as i32 + offset) as u32);
                                successors.push(*addr + format.len() as u32);
                            }
                        }
                    },
                    other => warn!("Unexpected instruction: {other:?}"),
                }
            }
            blocks.push(BasicBlock {
                start,
                end,
                instructions: instructions.into_iter().map(|(_, inst)| inst).collect(),
                successors,
            });
        }

        Ok(blocks)
    }
}
