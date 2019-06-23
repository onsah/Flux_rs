use super::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type HeapTable = Rc<RefCell<Table>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    table: HashMap<Value, Value>,
    // array: Vec<Value>,
}

impl Table {
    const NIL: Value = Value::Nil;

    pub fn new() -> Self {
        Table {
            table: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: Value, value: Value) {
        self.table.insert(key, value);
    }

    pub fn get(&self, key: &Value) -> &Value {
        self.table.get(key).unwrap_or(&Self::NIL)
    }
}
