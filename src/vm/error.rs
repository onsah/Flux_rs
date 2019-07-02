use crate::compiler::{BinaryInstr, Instruction, UnaryInstr};
use crate::vm::Value;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    TypeError,
    EmptyFrame,
    UnsupportedInstruction(Instruction),
    EmptyStack,
    UndefinedVariable { name: String },
    UnsupportedBinary { value: Value, op: BinaryInstr },
    IOError,
    InvalidFormat,
}
