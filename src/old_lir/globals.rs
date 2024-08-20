use crate::old_lir::Value;
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
                Global { name: String::from("Type"), value: Value::Type(Type::Type) },

                Global { name: String::from("Any"), value: Value::Type(Type::Any) },

                Global { name: String::from("None"), value: Value::Type(Type::None) },
                Global { name: String::from("Bool"), value: Value::Type(Type::Bool) },
                Global { name: String::from("Int"), value: Value::Type(Type::Int) },
                Global { name: String::from("Float"), value: Value::Type(Type::Float) },
            ]
        }
    }
}
