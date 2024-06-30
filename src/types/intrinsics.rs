use std::collections::HashMap;
use crate::types::{FunctionType, Type};

#[derive(Clone, Copy)]
pub enum IntrinsicFn {
    AddInt
}

pub struct IntrinsicLookup {
    map: HashMap<(Type, &'static str), (IntrinsicFn, FunctionType)>
}

impl IntrinsicLookup {
    pub fn new() -> Self {
        let map = HashMap::from([
            (
                (Type::Int, "+"),
                (IntrinsicFn::AddInt, FunctionType { params: vec![Type::Int, Type::Int], returns: Type::Int })
            )
        ]);

        Self { map }
    }

    pub fn find<'a>(&'a self, target_type: Type, name: &'a str) -> Option<&'a (IntrinsicFn, FunctionType)> {
        self.map.get(&(target_type, name))
    }
}

// TODO: This would be useful for the JIT compiler
//
// pub fn find_intrinsic(target_type: Type, name: &str) -> Option<(*const (), FunctionType)> {
//     match target_type {
//         Type::Any => panic!("No intrinsic functions present on Any - shouldn't be called"),
//         Type::None => None,
//         Type::Bool => todo!("Intrinsics on bool"),
//         Type::Int => unsafe {
//             match name {
//                 "+" => Some((std::mem::transmute(add_int), ADD_INT_TYPE.clone())),
//                 _ => None
//             }
//         }
//         Type::Float => todo!("Intrinsics on float"),
//         Type::Type => todo!("Intrinsics on type")
//     }
// }
//
// const ADD_INT_TYPE: FunctionType = FunctionType { params: vec![Type::Int, Type::Int], returns: Type::Int };
// extern "C" fn add_int(a: i64, b: i64) -> i64 { a + b }