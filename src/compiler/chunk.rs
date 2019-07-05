use super::Instruction;
use super::{CompileError, CompileResult};
use super::UpValueDesc;
use crate::vm::{Function, Value, PREDEFINED_CONSTANTS};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Chunk {
    instructions: Vec<Instruction>,
    constants: Vec<Value>,
    prototypes: Vec<FuncProto>,
}

#[derive(Clone, Debug)]
pub struct FuncProto {
    pub code_start: usize,
    pub args_len: u8,
    pub upvalues: Vec<UpValueDesc>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum JumpCondition {
    None,
    WhenTrue,
    WhenFalse,
}

impl Chunk {
    const MAX_CONST: usize = std::u8::MAX as usize;

    pub fn new() -> Self {
        Chunk {
            instructions: Vec::new(),
            constants: PREDEFINED_CONSTANTS.iter()
                .map(|(s, _)| Value::Embedded(s))
                .collect(),
            prototypes: Vec::new(),
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

    // Adds constant if not present
    pub fn add_constant(&mut self, constant: Value) -> CompileResult<u8> {
        let index = match &constant {
            Value::Str(string) => {
                if let Some(index) = self.has_string(string) {
                    self.push_instr(Instruction::Constant { index })?;
                    Ok(index)
                } else {
                    self.push_constant(constant)
                }
            }
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
        self.constants
            .iter()
            .enumerate()
            .find_map(|(i, s)| match s {
                Value::Str(s) => match **s == string {
                    true => Some(i as u8),
                    false => None,
                },
                Value::Embedded(s) => match *s == string {
                    true => Some(i as u8),
                    false => None,
                },
                _ => None,
            })
    }

    pub fn push_placeholder(&mut self) -> CompileResult<usize> {
        let index = self.instructions.len();
        self.push_instr(Instruction::Placeholder)?;
        Ok(index)
    }

    pub fn patch_placeholder(
        &mut self,
        index: usize,
        jump_offset: i8,
        jump_cond: JumpCondition,
    ) -> CompileResult<()> {
        let offset = jump_offset;
        let instr = match jump_cond {
            JumpCondition::None => Instruction::Jump { offset },
            JumpCondition::WhenTrue => Instruction::JumpIf {
                when_true: true,
                offset,
            },
            JumpCondition::WhenFalse => Instruction::JumpIf {
                when_true: false,
                offset,
            },
        };
        match self.instructions[index] {
            Instruction::Placeholder | Instruction::Jump { .. } | Instruction::JumpIf { .. } => {
                self.instructions[index] = instr;
                Ok(())
            }
            _ => Err(CompileError::WrongPatch(self.instructions[index])),
        }
    }

    pub fn push_proto(&mut self, code_start: usize, args_len: u8, upvalues: Vec<UpValueDesc>) -> usize {
        self.prototypes.push(FuncProto {
            code_start,
            args_len,
            upvalues
        });
        self.prototypes.len() - 1
    }

    pub fn prototypes(&self) -> &[FuncProto] {
        self.prototypes.as_slice()
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
