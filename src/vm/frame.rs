use super::value::UpValue;

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub(super) pc: usize,
    pub(super) stack_top: usize,
    pub(super) upvalues: Vec<UpValue>,
}

impl Frame {
    pub fn new(pc: usize, stack_top: usize, upvalues: Vec<UpValue>) -> Self {
        Frame {
            pc,
            stack_top,
            upvalues,
        }
    }

    pub fn stack_top(&self) -> usize {
        self.stack_top
    }
}

impl Default for Frame {
    fn default() -> Self {
        Frame {
            pc: 0,
            stack_top: 0,
            upvalues: Vec::new(),
        }
    }
}
