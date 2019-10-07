use super::{CompileError, CompileResult, ConstantTableStruct, Instruction};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledSource {
    pub chunk: Chunk,
    pub constant_table: Rc<ConstantTableStruct>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    instructions: Vec<Instruction>,
    imports: HashMap<String, Chunk>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FuncProto {
    pub args_len: u8,
    pub instructions: Box<[Instruction]>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum JumpCondition {
    None,
    WhenTrue,
    WhenFalse,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn push_instr(&mut self, instr: Instruction) -> CompileResult<()> {
        self.instructions.push(instr);
        Ok(())
    }

    pub fn add_import(&mut self, import: Chunk, name: String, name_index: u8) -> CompileResult<()> {
        if self.imports.contains_key(&name) {
            panic!("module '{}' is already imported", &name);
        }
        // let name_index = self.add_constant(name.clone().into())?;
        self.push_instr(Instruction::Import { name_index })?;
        self.imports.insert(name, import);
        Ok(())
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

    pub fn instructions(&self) -> &[Instruction] {
        self.instructions.as_slice()
    }

    pub fn instructions_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
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
            imports: HashMap::new(),
        }
    }
}
