#[derive(Clone, Copy)]
pub enum Type {
    Any,
    None,
    Bool,
    Int,
    Float,

    // TODO
    // Struct(ArenaRef<StructType>),
    // Interface(ArenaRef<InterfaceType>)
}

pub struct FunctionType {
    pub params: Vec<Type>,
    pub returns: Type
}
