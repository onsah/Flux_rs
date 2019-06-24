use super::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type HeapTable = Rc<RefCell<Table>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    table: HashMap<Value, Value>,
    array: Vec<Value>,
}

impl Table {
    const NIL: Value = Value::Nil;

    pub fn new() -> Self {
        Table {
            table: HashMap::new(),
            array: Vec::new(),
        }
    }

    pub fn from_array(array: Vec<Value>) -> Self {
        Table {
            table: HashMap::new(),
            array,
        }
    }

    pub fn set(&mut self, key: Value, value: Value) {
        self.table.insert(key, value);
    }

    pub fn get(&self, key: &Value) -> &Value {
        if let &Value::Int(i) = key {
            if i >= 0 {
                let i = i as usize;
                if self.array.len() < i {
                    return &self.array[i]
                }
            }
        }
        self.table.get(key).unwrap_or(&Self::NIL)
    }
}
