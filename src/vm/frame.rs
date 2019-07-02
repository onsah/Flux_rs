use super::Value;
use crate::compiler::{Chunk, Instruction};

#[derive(Copy, Clone, Debug)]
pub struct Frame {
    pub(super) pc: usize,
    pub(super) stack_top: usize,
}

impl Frame {
    pub fn new(pc: usize, stack_top: usize) -> Self {
        Frame { pc, stack_top }
    }

    pub fn stack_top(&self) -> usize {
        self.stack_top
    }
}
