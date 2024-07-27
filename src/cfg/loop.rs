use std::collections::HashSet;

use super::BasicBlock;

#[derive(Debug)]
pub struct Loop<'a> {
    header: &'a BasicBlock,
    blocks: Vec<&'a BasicBlock>,
    starts: HashSet<u32>
}

impl<'a> Loop<'a> {
    pub fn new(header: &'a BasicBlock) -> Loop<'a> {
        let mut starts = HashSet::new();
        starts.insert(header.start);
        Loop {
            header,
            blocks: vec![header],
            starts
        }
    }

    pub fn add_block(&mut self, block: &'a BasicBlock) {
        self.starts.insert(block.start);
        self.blocks.push(block);
    }

    pub fn contains(&self, block: &BasicBlock) -> bool {
        self.starts.contains(&block.start)
    }
}