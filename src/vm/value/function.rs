use std::hash::Hash;
use super::Value;
use crate::compiler::Instruction;

#[derive(Copy, Clone, Debug, Hash)]
pub struct Function {
    args_len: u8,
    code_start: usize,
}

impl Function {
    pub fn new(args_len: u8, code_start: usize) -> Self {
        Function {
            args_len,
            code_start,
        }
    }

    pub fn args_len(&self) -> u8 {
        self.args_len
    }

    pub fn code_start(&self) -> usize {
        self.code_start
    }
}

impl PartialEq for Function {
    fn eq(&self, rhs: &Function) -> bool {
        self.code_start == rhs.code_start
    }
}