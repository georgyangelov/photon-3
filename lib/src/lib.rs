mod old_value;

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

    // TODO: Optimization idea - another variant with no captures that points directly to the function
    Closure
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
            Any { typ: AnyT::Float, val } => unsafe { std::mem::transmute(val) },
            _ => panic!("Invalid value: expected {:?}, got {:?}", AnyT::Float, self.typ)
        }
    }

    pub unsafe fn trampoline_fn(self) -> extern "C" fn(*const Any) -> Any {
        let ptr_to_ptr_to_fn: *const extern "C" fn(*const Any) -> Any = std::mem::transmute(self.val);

        *ptr_to_ptr_to_fn
    }

    pub unsafe fn fn_0(self) -> extern "C" fn() -> Any {
        let ptr_to_ptr_to_fn: *const extern "C" fn() -> Any = std::mem::transmute(self.val);

        *ptr_to_ptr_to_fn
    }

    pub unsafe fn fn_1(self) -> extern "C" fn(Any) -> Any {
        let ptr_to_ptr_to_fn: *const extern "C" fn(Any) -> Any = std::mem::transmute(self.val);

        *ptr_to_ptr_to_fn
    }

    pub unsafe fn fn_2(self) -> extern "C" fn(Any, Any) -> Any {
        let ptr_to_ptr_to_fn: *const extern "C" fn(Any, Any) -> Any = std::mem::transmute(self.val);

        *ptr_to_ptr_to_fn
    }

    pub unsafe fn fn_3(self) -> extern "C" fn(Any, Any, Any) -> Any {
        let ptr_to_ptr_to_fn: *const extern "C" fn(Any, Any, Any) -> Any = std::mem::transmute(self.val);

        *ptr_to_ptr_to_fn
    }
}