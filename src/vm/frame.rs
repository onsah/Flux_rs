use super::Value;
use crate::compiler::{Chunk, Instruction};

#[derive(Copy, Clone, Debug)]
pub struct Frame {
    pub(super) pc: usize,
    pub(super) start: usize,
    pub(super) ret_adress: Option<usize>,
}

impl Frame {
    pub fn new(start: usize, ret_adress: Option<usize>) -> Self {
        Frame {
            pc: start,
            start,
            ret_adress,
        }
    }

    pub fn return_frame(self) -> Option<usize> {
        self.ret_adress
    }
}