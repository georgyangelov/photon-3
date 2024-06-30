#[derive(Clone, Copy, Debug, PartialEq)]
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

pub struct FunctionType {
    pub params: Vec<Type>,
    pub returns: Type
}
