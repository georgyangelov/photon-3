use std::rc::Rc;
use crate::compiler::mir::Function;

#[derive(Clone, Debug)]
pub enum Value {
    None,

    I8(i8),
    // I32(i32),
    I64(i64),
    F64(f64),

    Closure(Rc<Closure>)

    // String(String),

    // Struct()
}

#[derive(Debug)]
pub struct Closure {
    pub values: Vec<Value>,

    // TODO: Should actually be FunctionRef
    pub function: Function
}

impl Value {
    pub fn expect_i64(self) -> i64 {
        match self {
            Value::I64(value) => value,
            _ => todo!("Error handling")
        }
    }
}