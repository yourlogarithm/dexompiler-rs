mod block;

pub(crate) use block::BasicBlock;
use std::collections::{HashMap, HashSet};

use crate::{errors::{CallGraphError, InstructionError}, instruction::{Format, Instruction, Opcode}};

#[derive(Debug, Default)]
pub struct MethodControlFlowGraph {
    instructions: Vec<(u32, Instruction)>,
    switch_payload_origin: HashMap<u32, Vec<u32>>,
    switch_branch_targets: HashMap<u32, Vec<u32>>,
    leaders: HashSet<u32>,
}

impl MethodControlFlowGraph {
    pub fn target(op: Opcode, address: u32, format: &Format) -> Result<u32, CallGraphError> {
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
}
