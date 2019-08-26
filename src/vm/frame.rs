use super::value::{UpValue, FuncProtoRef};

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub(super) pc: usize,
    pub(super) proto: Option<FuncProtoRef>,
    pub(super) stack_top: usize,
    pub(super) upvalues: Vec<UpValue>,
}

impl Frame {
    pub fn new(pc: usize, proto: FuncProtoRef, stack_top: usize, upvalues: Vec<UpValue>) -> Self {
        Frame {
            pc,
            proto: Some(proto),
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
            proto: None,
            stack_top: 0,
            upvalues: Vec::new(),
        }
    }
}
