use std::mem::transmute;

/// The repr(C, i32) ensures that the layout of this enum is:
///
/// struct Value { type: i32, ... }
///
/// Make sure that when adding values, these are always at most 64-bits as we will pass the Value
/// structs as a tuple of (i32, i64).
///
/// See https://github.com/rust-lang/rfcs/blob/master/text/2195-really-tagged-unions.md
// #[derive(Clone, Debug)]
// #[repr(C, i32)]
// pub enum Value {
//     None,
//
//     Bool(bool),
//     // I32(i32),
//     I64(i64),
//     F64(f64),
//
//     // Closure(Rc<Closure>)
//
//     // String(String),
//
//     // Struct()
// }

// #[repr(transparent)]
// pub struct Ptr {
//     pub address: i32
// }

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueT {
    None,
    Bool,
    I64,
    F64,
    // Ptr
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueV {
    value: i64
}

impl ValueV {
    // #[inline]
    // pub unsafe fn unwrap_ptr(&self) -> Ptr {
    //     Ptr { address: self.value as i32 }
    // }

    pub unsafe fn to_literal(self) -> i64 {
        self.value
    }
}

impl ValueT {
    pub unsafe fn to_literal(self) -> i32 {
        transmute(self)
    }

    pub unsafe fn from_raw(t: i32, v: i64) -> (ValueT, ValueV) {
        (transmute(t), transmute(v))
    }

    pub fn none() -> (ValueT, ValueV) {
        (ValueT::None, ValueV { value: 0 })
    }

    pub fn bool(value: bool) -> (ValueT, ValueV) {
        (ValueT::Bool, ValueV { value: if value { 1 } else { 0 } })
    }

    pub fn i64(value: i64) -> (ValueT, ValueV) {
        (ValueT::I64, ValueV { value })
    }

    pub fn f64(value: f64) -> (ValueT, ValueV) {
        (ValueT::F64, ValueV { value: unsafe { transmute(value) } })
    }

    pub fn assert_none(self) {
        match self {
            ValueT::None => {},
            _ => panic!("Invalid value type, expected {:?}, got {:?}", ValueT::None, self)
        }
    }

    pub fn assert_i64(self, v: ValueV) -> i64 {
        match self {
            ValueT::I64 => v.value,
            _ => panic!("Invalid value type, expected {:?}, got {:?}", ValueT::I64, self)
        }
    }

    pub fn assert_f64(self, v: ValueV) -> f64 {
        match self {
            ValueT::F64 => unsafe { transmute(v.value) },
            _ => panic!("Invalid value type, expected {:?}, got {:?}", ValueT::F64, self)
        }
    }

    pub fn assert_bool(self, v: ValueV) -> bool {
        match self {
            ValueT::Bool => v.value != 0,
            _ => panic!("Invalid value type, expected {:?}, got {:?}", ValueT::Bool, self)
        }
    }
}