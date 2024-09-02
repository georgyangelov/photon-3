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

#[derive(Debug, Clone, Copy)]
pub enum IntrinsicFn {
    AddInt
}

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub returns: Type
}

// PERFORMANCE: Optimize to not create new objects every time
impl IntrinsicFn {
    pub fn signature(&self, arg_types: &[Type]) -> FunctionSignature {
        match self {
            IntrinsicFn::AddInt => FunctionSignature { params: vec![Type::Int, Type::Int], returns: Type::Int }
        }
    }
}
