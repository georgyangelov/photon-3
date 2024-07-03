use std::collections::HashMap;
use crate::mir;
use crate::types::{FunctionSignature, IntrinsicFn, Type};

// TODO: Better name? Maybe FunctionRegistry?
pub struct TypeRegistry {
    map: HashMap<(Type, String), ResolvedFn>
}

impl TypeRegistry {
    pub fn new() -> Self {
        let map = HashMap::from([
            (
                (Type::Int, String::from("+")),
                ResolvedFn::Intrinsic(IntrinsicFn::AddInt)
            )
        ]);

        Self { map }
    }

    pub fn register(&mut self, target_type: Type, name: String, func: mir::FunctionRef) {
        self.map.insert((target_type, name), ResolvedFn::Function(func));
    }

    // TODO: This should receive the argument types and return the concrete argument types
    pub fn resolve(&self, target_type: Type, name: &str) -> Option<ResolvedFn> {
        self.map.get(&(target_type, String::from(name))).cloned()
    }
}

#[derive(Clone, Copy)]
pub enum ResolvedFn {
    Intrinsic(IntrinsicFn),
    Function(mir::FunctionRef)
}

// TODO: Optimize to not create new objects every time
pub fn intrinsic_signature(intrinsic: IntrinsicFn) -> FunctionSignature {
    match intrinsic {
        IntrinsicFn::AddInt => FunctionSignature { params: vec![Type::Int, Type::Int], returns: Type::Int }
    }
}