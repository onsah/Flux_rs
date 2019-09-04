use super::Instruction;
use super::UpValueDesc;
use super::{CompileError, CompileResult};
use crate::vm::lib::constant_names;
use crate::vm::{Value, FuncProtoRef, UpValue};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    instructions: Vec<Instruction>,
    constants: Vec<Value>,
    prototypes: Vec<FuncProtoRef>,
    imports: HashMap<String, Chunk>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FuncProto {
    pub args_len: u8,
    pub upvalues: Vec<(usize, Rc<RefCell<UpValue>>)>,
    pub instructions: Box<[Instruction]>,
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
        Self::default()
    }

    #[inline]
    fn push_instr(&mut self, instr: Instruction) -> CompileResult<()> {
        self.instructions.push(instr);
        Ok(())
    }

    pub fn add_import(&mut self, import: Chunk, name: String) -> CompileResult<()> {
        if self.imports.contains_key(&name) {
            panic!("module '{}' is already imported", &name);
        }
        let name_index = self.add_constant(name.clone().into())?;
        self.push_instr(Instruction::Import { name_index })?;
        self.imports.insert(name, import);
        Ok(())
    }

    // Adds constant if not present
    pub fn add_constant(&mut self, constant: Value) -> CompileResult<u8> {
        let index = match &constant {
            Value::Str(string) => {
                if let Some(index) = self.has_string(string) {
                    Ok(index)
                } else {
                    self.push_constant(constant)
                }
            }
            _ => self.push_constant(constant),
        }?;
        Ok(index)
    }

    #[inline]
    pub fn push_constant(&mut self, constant: Value) -> CompileResult<u8> {
        if self.constants.len() >= Self::MAX_CONST {
            Err(CompileError::TooManyConstants)
        } else {
            self.constants.push(constant);
            let index = (self.constants.len() - 1) as u8;
            // self.push_instr(Instruction::Constant { index })?;
            Ok(index)
        }
    }

    pub fn has_string(&self, string: &str) -> Option<u8> {
        self.constants
            .iter()
            .enumerate()
            .find_map(|(i, s)| match s {
                Value::Str(s) => {
                    if **s == string {
                        Some(i as u8)
                    } else {
                        None
                    }
                }
                Value::Embedded(s) => {
                    if *s == string {
                        Some(i as u8)
                    } else {
                        None
                    }
                }
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

    pub fn push_proto(
        &mut self,
        args_len: u8,
        upvalues: Vec<(usize, Rc<RefCell<UpValue>>)>,
        instructions: Vec<Instruction>,
    ) -> usize {
        self.prototypes.push(Rc::new(FuncProto {
            args_len,
            upvalues,
            instructions: instructions.into_boxed_slice(),
        }));
        self.prototypes.len() - 1
    }

    pub fn prototypes(&self) -> &[FuncProtoRef] {
        self.prototypes.as_slice()
    }

    pub fn instructions(&self) -> &[Instruction] {
        self.instructions.as_slice()
    }

    pub fn instructions_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
    }

    pub fn constants(&self) -> &[Value] {
        self.constants.as_slice()
    }

    pub fn take_imports(&mut self) -> HashMap<String, Chunk> {
        std::mem::replace(&mut self.imports, HashMap::new())
    }

    pub fn imports(&mut self) -> &mut HashMap<String, Chunk> {
        &mut self.imports
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            instructions: Vec::new(),
            constants: constant_names().collect(),
            prototypes: Vec::new(),
            imports: HashMap::new(),
        }
    }
}
