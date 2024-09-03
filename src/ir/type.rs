use crate::ir;

// TODO: Make this faster by using arena allocation instead of cloning
#[derive(Clone, Debug)]
pub enum Type {
    Any,
    None,
    Bool,
    Int,
    Float,
    Type,

    StaticClosure(ir::FunctionRef, Vec<StaticCapture>)

    // TODO: We'll also need an interface type for functions which the closures can be assigned to
    // Closure(ir::FunctionRef)

    // TODO
    // Struct(ArenaRef<StructType>),
    // Interface(ArenaRef<InterfaceType>)
}

impl Type {
    pub fn is_any(&self) -> bool {
        match self {
            Type::Any => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone)]
pub struct StaticCapture {
    pub typ: Type,
    pub comptime_value: Option<ir::Value>
}
