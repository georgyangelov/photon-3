use std::rc::Rc;
use crate::mir;

#[derive(Clone, Debug)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Closure(mir::FunctionRef, Rc<Vec<Value>>)
}

impl Value {
    pub fn assert_none(self) {
        match self {
            Value::None => {},
            _ => panic!("Invalid value: expected None, got {:?}", self)
        }
    }

    pub fn assert_bool(&self) -> bool {
        match self {
            Value::Bool(value) => *value,
            _ => panic!("Invalid value: expected Bool, got {:?}", self)
        }
    }

    pub fn assert_int(&self) -> i64 {
        match self {
            Value::Int(value) => *value,
            _ => panic!("Invalid value: expected Int, got {:?}", self)
        }
    }

    pub fn assert_float(&self) -> f64 {
        match self {
            Value::Float(value) => *value,
            _ => panic!("Invalid value: expected Float got {:?}", self)
        }
    }

    pub fn assert_closure(&self) -> (mir::FunctionRef, &Vec<Value>) {
        match self {
            Value::Closure(func_ref, value) => (*func_ref, value.as_ref()),
            _ => panic!("Invalid value: expected Closure, got {:?}", self)
        }
    }
}