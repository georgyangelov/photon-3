use crate::ir;

#[derive(Clone, Debug)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Type(ir::Type),
    StaticClosure(Vec<Value>)
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

    pub fn assert_type(&self) -> &ir::Type {
        match self {
            Value::Type(value) => value,
            _ => panic!("Invalid value: expected Type got {:?}", self)
        }
    }

    pub fn assert_static_closure(&self) -> &Vec<Value> {
        match self {
            Value::StaticClosure(value) => value,
            _ => panic!("Invalid value: expected Closure, got {:?}", self)
        }
    }

    pub fn into_static_closure(self) -> Vec<Value> {
        match self {
            Value::StaticClosure(value) => value,
            _ => panic!("Invalid value: expected Closure, got {:?}", self)
        }
    }

    // TODO: Do I need this, is this correct?
    pub fn type_of(&self) -> ir::Type {
        match self {
            Value::None => ir::Type::None,
            Value::Bool(_) => ir::Type::Bool,
            Value::Int(_) => ir::Type::Int,
            Value::Float(_) => ir::Type::Float,
            Value::Type(_) => ir::Type::Type,
            Value::StaticClosure(_) => todo!("Cannot get type of a static closure"),
        }
    }
}

impl From<Value> for i64 {
    fn from(value: Value) -> Self { value.assert_int() }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self { value.assert_bool() }
}

impl From<Value> for f64 {
    fn from(value: Value) -> Self { value.assert_float() }
}