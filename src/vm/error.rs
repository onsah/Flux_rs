use crate::compiler::{BinaryInstr, Instruction};
use crate::vm::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    TypeError,
    EmptyFrame,
    UnsupportedInstruction(Instruction),
    EmptyStack,
    UndefinedVariable { name: String },
    UnsupportedBinary { value: Value, op: BinaryInstr },
    IOError,
    InvalidFormat,
    WrongNumberOfArgs { expected: u8, found: u8 },
    DivideByZero
}
