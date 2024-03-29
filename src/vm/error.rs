use crate::compiler::{BinaryInstr, Instruction};
use crate::error::FluxError;
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
    ExpectedArgsAtLeast(u8),
    DivideByZero,
    AssertionFailed(Value),
    ImportError { error: FluxError, module: String },
}
