use std::rc::Rc;
use crate::mir;

#[derive(Clone, Debug)]
pub enum Value {
    None,

    Bool(bool),
    // I32(i32),
    I64(i64),
    F64(f64),

    Closure(Rc<Closure>)

    // String(String),

    // Struct()
}

#[derive(Debug)]
pub struct Closure {
    pub function_ref: mir::FunctionRef,
    pub values: Vec<Value>
}

impl Value {
    pub fn expect_i64(self) -> i64 {
        match self {
            Value::I64(value) => value,
            _ => todo!("Error handling")
        }
    }

    pub fn expect_bool(self) -> bool {
        match self {
            Value::Bool(value) => value,
            _ => todo!("Error handling")
        }
    }
}