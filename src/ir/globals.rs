use crate::ir::{Type, Value};

pub struct Globals {
    pub globals: Vec<Global>
}

pub struct Global {
    pub name: String,
    pub value: Value,
    pub comptime: bool
}

impl Globals {
    pub fn new() -> Self {
        Globals {
            globals: vec![
                Global { name: String::from("Type"), value: Value::Type(Type::Type), comptime: true },

                Global { name: String::from("Any"), value: Value::Type(Type::Any), comptime: true },

                Global { name: String::from("None"), value: Value::Type(Type::None), comptime: true },
                Global { name: String::from("Bool"), value: Value::Type(Type::Bool), comptime: true },
                Global { name: String::from("Int"), value: Value::Type(Type::Int), comptime: true },
                Global { name: String::from("Float"), value: Value::Type(Type::Float), comptime: true },
            ]
        }
    }
}
