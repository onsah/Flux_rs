use super::Instruction;
use crate::vm::Value;
use super::{CompileResult, CompileError};

#[derive(Clone, Debug)]
pub struct Chunk {
    instructions: Vec<Instruction>,
    constants: Vec<Value>,
}

impl Chunk {
    const MAX_CONST: usize = std::u8::MAX as usize;

    pub fn new() -> Self {
        Chunk {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }

    #[inline]
    pub fn push_instr(&mut self, instr: Instruction) -> CompileResult<()> {
        if self.instructions().len() < Self::MAX_CONST {
            self.instructions.push(instr);
            Ok(())
        } else {
            Err(CompileError::TooManyConstants)
        }
    }

    pub fn add_constant(&mut self, constant: Value) -> CompileResult<u8> {
        let index = match &constant {
            Value::Str(string) => {
                if let Some(index) = self.has_string(string) {
                    self.push_instr(Instruction::Constant { index })?;
                    Ok(index)
                } else {
                    self.push_constant(constant)
                }
            },
            _ => self.push_constant(constant),
        }?;
        self.pop_constant();
        Ok(index)
    }

    #[inline]
    pub fn push_constant(&mut self, constant: Value) -> CompileResult<u8> {
        self.constants.push(constant);
        let index = (self.constants.len() - 1) as u8;
        self.push_instr(Instruction::Constant { index })?;
        Ok(index)
    }

    pub fn has_string(&self, string: &str) -> Option<u8> {
        self.constants.iter()
            .enumerate()
            .find_map(|(i, s)| match s {
                Value::Str(s) => {
                    match **s == string {
                        true => Some(i as u8),
                        false => None,
                    }
                }
                _ => None
            }
        )
    }

    #[inline]
    fn pop_constant(&mut self) {
        self.instructions.pop().unwrap();
    }

    pub fn instructions(&self) -> &[Instruction] {
        self.instructions.as_slice()
    }

    pub fn constants(&self) -> &[Value] {
        self.constants.as_slice()
    }
}