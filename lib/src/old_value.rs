// use std::intrinsics::transmute;
//
// #[repr(i32)]
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub enum ValueT {
//     None,
//     Bool,
//     I64,
//     F64,
//     Array,
//     // Ptr
// }
//
// #[repr(transparent)]
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct ValueV {
//     value: i64
// }
//
// #[repr(C, packed)]
// #[derive(Debug)]
// pub struct ArrayPtr {
//     pub size: i32,
//     pub ptr: i32
// }
//
// impl ValueV {
//     pub unsafe fn to_literal(self) -> i64 {
//         self.value
//     }
// }
//
// impl ValueT {
//     pub unsafe fn to_literal(self) -> i32 {
//         transmute(self)
//     }
//
//     pub unsafe fn from_raw(t: i32, v: i64) -> (ValueT, ValueV) {
//         (transmute(t), transmute(v))
//     }
//
//     pub fn none() -> (ValueT, ValueV) {
//         (ValueT::None, ValueV { value: 0 })
//     }
//
//     pub fn bool(value: bool) -> (ValueT, ValueV) {
//         (ValueT::Bool, ValueV { value: if value { 1 } else { 0 } })
//     }
//
//     pub fn i64(value: i64) -> (ValueT, ValueV) {
//         (ValueT::I64, ValueV { value })
//     }
//
//     pub fn f64(value: f64) -> (ValueT, ValueV) {
//         (ValueT::F64, ValueV { value: unsafe { transmute(value) } })
//     }
//
//     pub fn array(ptr: ArrayPtr) -> (ValueT, ValueV) {
//         (ValueT::Array, ValueV { value: unsafe { transmute(ptr) } })
//     }
//
//     pub fn assert_none(self) {
//         match self {
//             ValueT::None => {},
//             _ => panic!("Invalid value type, expected {:?}, got {:?}", ValueT::None, self)
//         }
//     }
//
//     pub fn assert_i64(self, v: ValueV) -> i64 {
//         match self {
//             ValueT::I64 => v.value,
//             _ => panic!("Invalid value type, expected {:?}, got {:?}", ValueT::I64, self)
//         }
//     }
//
//     pub fn assert_f64(self, v: ValueV) -> f64 {
//         match self {
//             ValueT::F64 => unsafe { transmute(v.value) },
//             _ => panic!("Invalid value type, expected {:?}, got {:?}", ValueT::F64, self)
//         }
//     }
//
//     pub fn assert_bool(self, v: ValueV) -> bool {
//         match self {
//             ValueT::Bool => v.value != 0,
//             _ => panic!("Invalid value type, expected {:?}, got {:?}", ValueT::Bool, self)
//         }
//     }
//
//     pub fn assert_array(self, v: ValueV) -> ArrayPtr {
//         match self {
//             ValueT::Array => unsafe { transmute(v.value) },
//             _ => panic!("Invalid value type, expected {:?}, got {:?}", ValueT::Array, self)
//         }
//     }
//
//     pub fn unwrap_array(self, v: ValueV) -> ArrayPtr {
//         unsafe { transmute(v.value) }
//     }
// }