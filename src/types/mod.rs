mod intrinsics;

pub use intrinsics::*;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    Any,
    None,
    Bool,
    Int,
    Float,
    Type,

    // TODO
    // Struct(ArenaRef<StructType>),
    // Interface(ArenaRef<InterfaceType>)
}

#[derive(Clone)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub returns: Type
}
