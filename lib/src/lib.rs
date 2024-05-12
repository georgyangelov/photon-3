mod old_value;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Value {
    typ: ValueT,
    val: i64
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum ValueT {
    None,
    Bool,
    Int,
    Float
}

impl Value {
    pub fn none() -> Self {
        Value { typ: ValueT::None, val: 0 }
    }

    pub fn bool(value: bool) -> Self {
        Value {
            typ: ValueT::Bool,
            val: if value { 1 } else { 0 }
        }
    }

    pub fn int(value: i64) -> Self {
        Value {
            typ: ValueT::Int,
            val: value
        }
    }

    pub fn float(value: f64) -> Self {
        Value {
            typ: ValueT::Float,
            val: unsafe { std::mem::transmute(value) }
        }
    }

    pub fn into_raw(self) -> (i32, i64) {
        (unsafe { std::mem::transmute(self.typ) }, self.val)
    }

    pub fn assert_none(self) {
        match self {
            Value { typ: ValueT::None, .. } => {},
            _ => panic!("Invalid value: expected {:?}, got {:?}", ValueT::None, self.typ)
        }
    }

    pub fn assert_bool(self) -> bool {
        match self {
            Value { typ: ValueT::Bool, val } => val != 0,
            _ => panic!("Invalid value: expected {:?}, got {:?}", ValueT::Bool, self.typ)
        }
    }

    pub fn assert_int(self) -> i64 {
        match self {
            Value { typ: ValueT::Int, val } => val,
            _ => panic!("Invalid value: expected {:?}, got {:?}", ValueT::Int, self.typ)
        }
    }

    pub fn assert_float(self) -> f64 {
        match self {
            Value { typ: ValueT::Float, val } => unsafe { std::mem::transmute(val) },
            _ => panic!("Invalid value: expected {:?}, got {:?}", ValueT::Float, self.typ)
        }
    }
}