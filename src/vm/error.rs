use crate::compiler::{Instruction, BinaryInstr, UnaryInstr};
use crate::vm::Value;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    TypeError,
    EmptyFrame,
    UnsupportedInstruction(Instruction),
    EmptyStack,
    UnsupportedBinary {
        value: Value,
        op: BinaryInstr,
    }
}