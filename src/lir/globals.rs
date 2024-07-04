use crate::lir::Value;
use crate::types::Type;

pub struct Globals {
    pub globals: Vec<Global>
}

pub struct Global {
    pub name: String,
    pub value: Value
}

impl Globals {
    pub fn new() -> Self {
        Globals {
            globals: vec![
                Global { name: String::from("Int"), value: Value::Type(Type::Int) }
            ]
        }
    }
}
