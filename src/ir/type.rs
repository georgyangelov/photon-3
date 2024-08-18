use crate::ir;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    Any,
    None,
    Bool,
    Int,
    Float,
    Type,

    // TODO: We'll also need an interface type for functions which the closures can be assigned to
    Closure(ir::FunctionTemplateRef)

    // TODO
    // Struct(ArenaRef<StructType>),
    // Interface(ArenaRef<InterfaceType>)
}

#[derive(Debug, Clone, Copy)]
pub enum IntrinsicFn {
    AddInt
}

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub returns: Type
}
