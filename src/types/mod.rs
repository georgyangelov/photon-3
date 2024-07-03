mod type_registry;

pub use type_registry::*;
use crate::lir;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    Any,
    None,
    Bool,
    Int,
    Float,
    Type,

    // TODO: We'll also need an interface type for functions which the closures can be assigned to
    Closure(lir::FunctionRef)

    // TODO
    // Struct(ArenaRef<StructType>),
    // Interface(ArenaRef<InterfaceType>)
}

#[derive(Clone, Copy)]
pub enum IntrinsicFn {
    AddInt
}

#[derive(Clone)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub returns: Type
}
