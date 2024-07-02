use std::{cell::RefCell, rc::Rc};

use crate::instruction::Instruction;

#[derive(Debug, Default)]
pub struct BasicBlock {
    pub start: u32,
    pub end: u32,
    pub instructions: Vec<Instruction>,
    pub successors: Vec<Rc<RefCell<BasicBlock>>>,
}