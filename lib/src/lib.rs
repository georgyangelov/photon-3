mod old_value;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Value {
    pub typ: ValueT,
    val: i64
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ValueT {
    None,
    Bool,
    Int,
    Float,

    // TODO: Optimization idea - another variant with no captures that points directly to the function
    Closure
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

    pub fn assert_closure(self) -> *mut u8 {
        match self {
            Value { typ: ValueT::Float, val } => unsafe { std::mem::transmute(val) },
            _ => panic!("Invalid value: expected {:?}, got {:?}", ValueT::Float, self.typ)
        }
    }

    pub unsafe fn fn_0(self) -> extern "C" fn() -> Value {
        let ptr_to_ptr_to_fn: *const extern "C" fn() -> Value = std::mem::transmute(self.val);

        *ptr_to_ptr_to_fn
    }

    pub unsafe fn fn_1(self) -> extern "C" fn(Value) -> Value {
        let ptr_to_ptr_to_fn: *const extern "C" fn(Value) -> Value = std::mem::transmute(self.val);

        *ptr_to_ptr_to_fn
    }

    pub unsafe fn fn_2(self) -> extern "C" fn(Value, Value) -> Value {
        let ptr_to_ptr_to_fn: *const extern "C" fn(Value, Value) -> Value = std::mem::transmute(self.val);

        *ptr_to_ptr_to_fn
    }

    pub unsafe fn fn_3(self) -> extern "C" fn(Value, Value, Value) -> Value {
        let ptr_to_ptr_to_fn: *const extern "C" fn(Value, Value, Value) -> Value = std::mem::transmute(self.val);

        *ptr_to_ptr_to_fn
    }
}