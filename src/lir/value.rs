use std::rc::Rc;
use crate::lir;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Type(Type),

    // PERFORMANCE: Potential to optimize performance by packing this?
    Closure(lir::FunctionRef, Rc<Vec<Value>>)
}

impl Value {
    pub fn assert_none(&self) {
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

    pub fn assert_type(&self) -> Type {
        match self {
            Value::Type(value) => *value,
            _ => panic!("Invalid value: expected Type got {:?}", self)
        }
    }

    pub fn assert_closure(&self) -> (lir::FunctionRef, &Vec<Value>) {
        match self {
            Value::Closure(func_ref, value) => (*func_ref, value.as_ref()),
            _ => panic!("Invalid value: expected Closure, got {:?}", self)
        }
    }

    pub fn type_of(&self) -> Type {
        match self {
            Value::None => Type::None,
            Value::Bool(_) => Type::Bool,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::Type(_) => Type::Type,
            Value::Closure(_, _) => todo!("Support types of closures"),
        }
    }
}