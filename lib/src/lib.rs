#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Any {
    pub typ: AnyT,
    val: i64
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AnyT {
    None,
    Bool,
    Int,
    Float,

    AnyT,
    TypeT,
    NoneT,
    BoolT,
    IntT,
    FloatT,

    // TODO: Support compound types
    // StructT,
    // ClosureT,
    // InterfaceT,

    // TODO: Optimization idea - another variant with no captures that points directly to the function
    Closure,
    FunctionPtr
}

impl AnyT {
    pub fn into_raw(self) -> i32 {
        unsafe { std::mem::transmute(self) }
    }
}

impl Any {
    pub fn none() -> Self {
        Any { typ: AnyT::None, val: 0 }
    }

    pub fn bool(value: bool) -> Self {
        Any {
            typ: AnyT::Bool,
            val: if value { 1 } else { 0 }
        }
    }

    pub fn int(value: i64) -> Self {
        Any {
            typ: AnyT::Int,
            val: value
        }
    }

    pub fn float(value: f64) -> Self {
        Any {
            typ: AnyT::Float,
            val: unsafe { std::mem::transmute(value) }
        }
    }

    pub const fn any_type() -> Self {
        Any { typ: AnyT::AnyT, val: 0 }
    }

    pub const fn type_type() -> Self {
        Any { typ: AnyT::TypeT, val: 0 }
    }

    pub const fn none_type() -> Self {
        Any { typ: AnyT::NoneT, val: 0 }
    }

    pub const fn bool_type() -> Self {
        Any { typ: AnyT::BoolT, val: 0 }
    }

    pub const fn int_type() -> Self {
        Any { typ: AnyT::BoolT, val: 0 }
    }

    pub const fn float_type() -> Self {
        Any { typ: AnyT::FloatT, val: 0 }
    }

    pub fn into_raw(self) -> (i32, i64) {
        (unsafe { std::mem::transmute(self.typ) }, self.val)
    }

    pub fn assert_none(self) {
        match self {
            Any { typ: AnyT::None, .. } => {},
            _ => panic!("Invalid value: expected {:?}, got {:?}", AnyT::None, self.typ)
        }
    }

    pub fn assert_bool(self) -> bool {
        match self {
            Any { typ: AnyT::Bool, val } => val != 0,
            _ => panic!("Invalid value: expected {:?}, got {:?}", AnyT::Bool, self.typ)
        }
    }

    pub fn assert_int(self) -> i64 {
        match self {
            Any { typ: AnyT::Int, val } => val,
            _ => panic!("Invalid value: expected {:?}, got {:?}", AnyT::Int, self.typ)
        }
    }

    pub fn assert_float(self) -> f64 {
        match self {
            Any { typ: AnyT::Float, val } => unsafe { std::mem::transmute(val) },
            _ => panic!("Invalid value: expected {:?}, got {:?}", AnyT::Float, self.typ)
        }
    }

    pub fn assert_closure(self) -> *mut u8 {
        match self {
            Any { typ: AnyT::Closure, val } => unsafe { std::mem::transmute(val) },
            _ => panic!("Invalid value: expected {:?}, got {:?}", AnyT::Closure, self.typ)
        }
    }

    pub unsafe fn trampoline_fn(self) -> extern "C" fn(*const Any) -> Any {
        let ptr_to_ptr_to_fn: extern "C" fn(*const Any) -> Any = std::mem::transmute(self.val);

        ptr_to_ptr_to_fn
    }

    pub unsafe fn trampoline_closure(self) -> (extern "C" fn(*const Any, *const u8) -> Any, *const u8) {
        let val = self.val;
        let ptr_to_ptr_to_fn: *const extern "C" fn(*const Any, *const u8) -> Any = std::mem::transmute(val);

        (*ptr_to_ptr_to_fn, std::mem::transmute(val))
    }
}
