use super::Instruction;
use super::{CompileError, CompileResult};
use crate::vm::{Value, Function};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Chunk {
    instructions: Vec<Instruction>,
    constants: Vec<Value>,
    locals: Vec<Local>,
    depth: u8,
    function_depth: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Local {
    name: String,
    depth: u8,
    function: Option<u8>,
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
            constants: Vec::new(),
            locals: Vec::new(),
            depth: 0,
            function_depth: Vec::new(),
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
            },
            _ => Err(CompileError::WrongPatch(self.instructions[index])),
        }
    }

    pub fn resolve_local(&self, name: &str) -> Option<(usize, Local)> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, l)| match l.name == name {
                true => {
                    if l.in_function() {
                        self.resolve_fn_local(name)
                    } else {
                        Some((i, l.clone()))
                    }
                },
                false => None,
            })
    }

    fn resolve_fn_local(&self, name: &str) -> Option<(usize, Local)> {
        self.locals
            .iter()
            .filter(|l| l.depth >= *self.function_depth.last().unwrap())
            .enumerate()
            .find_map(|(i, l)| match l.name == name {
                true => Some((i, l.clone())),
                false => None
            })
    }

    pub fn push_local(&mut self, name: String) {
        self.locals.push(Local {
            name,
            depth: self.depth,
            function: match self.function_depth.len() {
                0 => None,
                i => Some(i as u8 - 1),
            }
        })
    }

    pub fn scope_incr(&mut self) {
        self.depth += 1
    }

    pub fn enter_function(&mut self) {
        self.scope_incr();
        self.function_depth.push(self.depth)
    }

    pub fn scope_decr(&mut self) -> usize {
        self.depth -= 1;
        let mut pop_count = 0;
        while self.locals.last().is_some() && self.locals.last().unwrap().depth > self.depth {
            self.locals.pop().unwrap();
            pop_count += 1;
        }
        pop_count
    }

    pub fn exit_function(&mut self) -> usize {
        self.function_depth.pop().unwrap();
        self.scope_decr()
    }
    
    pub fn is_in_function(&self) -> bool {
        !self.function_depth.is_empty()
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

impl Local {
    pub fn in_function(&self) -> bool {
        self.function.is_some()
    }
}