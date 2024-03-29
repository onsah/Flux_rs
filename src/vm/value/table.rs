use super::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type TableRef = Rc<RefCell<Table>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    table: HashMap<Value, Value>,
    array: Vec<(Value, Value)>,
}

impl Table {
    const NIL: Value = Value::Nil;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn shared(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    pub fn from_array(array: Vec<(Value, Value)>) -> Self {
        Table {
            table: HashMap::new(),
            array,
        }
    }

    pub fn from_map(table: HashMap<Value, Value>) -> Self {
        Table {
            table,
            array: vec![],
        }
    }

    pub fn set(&mut self, key: Value, value: Value) {
        self.table.insert(key, value);
    }

    pub fn get(&self, key: &Value) -> &Value {
        if let Some(i) = key.convert_int() {
            if i >= 0 {
                let i = i as usize;
                if self.array.len() > i {
                    return &self.array[i].1;
                }
            }
        }
        self.table.get(key).unwrap_or(&Self::NIL)
    }

    pub fn klass(&self) -> &Value {
        self.get(&Value::Embedded("__class__"))
    }

    pub fn pairs(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.table
            .iter()
            .chain(self.array.iter().map(|(v1, v2)| (v1, v2)))
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.array.iter().map(|(_, v)| v).chain(self.table.values())
    }
}

impl Default for Table {
    fn default() -> Self {
        Table {
            table: HashMap::new(),
            array: Vec::new(),
        }
    }
}
