#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    /*pop two values then apply operation */
    Bin(BinaryInstr),
    /*pop single value then apply operation */
    Unary(UnaryInstr),
    /*push 'nil' to stack */
    Nil,
    /*push 'true' to stack */
    True,
    /*push 'false' to stack */
    False,
    /*push a constant from constant pool */
    Constant { index: u8 },
    /*pop value from stack and create a global variable*/
    // DefGlobal { index: u8 },
    /*pop value from stack and set it to the global variable */
    SetGlobal { index: u8 },
    /*push the value of global to stack*/
    GetGlobal { index: u8 },
    /*pop the table then push the value of the key in the constant pool */
    GetFieldImm { index: u8 },
    /*pop the key and table then push the value from the table */
    GetField,
    /*pop the value then pop the table object then get the field name from pool then set the value to table*/
    SetFieldImm { index: u8 },
    /**
     * Pop the table, pop the key then pop the value then set the value to the respective key
     */
    SetField,
    /*Simply pop the top value from stack */
    Pop,
    /*Pop the value and return */
    Return,
    /*Pop len values and push a tuple */
    Tuple { len: u8 },
    /* Create a table with values */
    InitTable { len: u8, has_keys: bool },
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
    Ne
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryInstr {
    Negate,
    Not
}