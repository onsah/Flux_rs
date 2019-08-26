#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction {
    /*pop two values then apply operation */
    Bin(BinaryInstr),
    /*pop single value then apply operation */
    Unary(UnaryInstr),
    /*push 'nil' to stack */
    Nil,
    Unit,
    /*push 'true' to stack */
    True,
    /*push 'false' to stack */
    False,
    /*push a constant from constant pool */
    Constant {
        index: u8,
    },
    /*pop value from stack and create a global variable*/
    // DefGlobal { index: u8 },
    /*pop value from stack and set it to the global variable */
    SetGlobal {
        index: u8,
    },
    /*push the value of global to stack*/
    GetGlobal {
        index: u8,
    },
    /* These instructions are redundant because they are equivalent to SetFnLocal with top call frame */
    SetLocal {
        index: u16,
        frame: u8,
    },
    GetLocal {
        index: u16,
        frame: u8,
    },
    /*Peek the table then get the value using key and push it to the stack */
    GetMethodImm {
        index: u8,
        table_stack_index: u8,
    },
    /*pop the table then get the value using key and push to stack */
    GetFieldImm {
        index: u8,
    },
    /*pop the key and table then push the value from the table */
    GetField,
    SetFieldImm {
        index: u8,
    },
    /** pop the value then pop the table then set the key to the value
     * SetFieldImm { key: u16 }
     */
    /**
     * Pop the table, pop the key then pop the value then set the value to the respective key
     */
    SetField,
    /*Simply pop the top value from stack */
    Pop,
    /*Pop the value and return */
    Return {
        return_value: bool,
    },
    /*Pop len values and push a tuple */
    Tuple {
        len: u8,
    },
    /* Create a table with values */
    InitTable {
        len: u16,
        has_keys: bool,
    },
    /* Pop value if truth value matches with 'when_true' then branch */
    JumpIf {
        when_true: bool,
        offset: i8,
    },
    /* Directly jump */
    Jump {
        offset: i8,
    },
    /* Placeholder for patching jumps */
    Placeholder,
    Print,
    FuncDef {
        proto_index: usize,
    },
    Call {
        args_len: u8,
    },
    GetUpval {
        index: u16,
    },
    // SetUpval
    CloseUpval {
        index: u8,
    },
    Integer(i32),
    // Run the file then push the global table to stack
    Import {
        name_index: u8,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinaryInstr {
    Add,
    Sub,
    Mul,
    Div,
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UnaryInstr {
    Negate,
    Not,
}

impl BinaryInstr {
    pub fn is_arithmetic(self) -> bool {
        match self {
            BinaryInstr::Add | BinaryInstr::Sub | BinaryInstr::Mul | BinaryInstr::Div => true,
            _ => false,
        }
    }
}
