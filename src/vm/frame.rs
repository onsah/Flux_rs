use super::value::UpValue;

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub(super) pc: usize,
    pub(super) proto_index: Option<usize>,
    pub(super) stack_top: usize,
    pub(super) upvalues: Vec<UpValue>,
}

impl Frame {
    pub fn new(pc: usize, proto_index: usize, stack_top: usize, upvalues: Vec<UpValue>) -> Self {
        Frame {
            pc,
            proto_index: Some(proto_index),
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
            proto_index: None,
            stack_top: 0,
            upvalues: Vec::new(),
        }
    }
}
