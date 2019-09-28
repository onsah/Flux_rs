use super::value::{UserFunction, FuncProtoRef};

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub(super) pc: usize,
    pub(super) stack_top: usize,
    function: Option<UserFunction>,
}

impl Frame {
    pub fn new(pc: usize, function: UserFunction, stack_top: usize) -> Self {
        Frame {
            pc,
            function: Some(function),
            stack_top,
        }
    }

    pub fn stack_top(&self) -> usize {
        self.stack_top
    }

    pub fn proto(&self) -> Option<&FuncProtoRef> {
        self.function.as_ref().map(|f| f.proto_ref())
    }

    pub fn function(&self) -> Option<&UserFunction> {
        self.function.as_ref()
    }
}

impl Default for Frame {
    fn default() -> Self {
        Frame {
            pc: 0,
            function: None,
            stack_top: 0,
        }
    }
}
