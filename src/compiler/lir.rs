use crate::compiler::lexical_scope::{Capture, GlobalSlotRef, LocalSlotRef};

// pub struct LIR {
//     pub node: Node,
//     pub typ: Type
// }

pub enum Node {
    GlobalGet(GlobalSlotRef),
    GlobalSet(GlobalSlotRef, Box<Node>),

    LocalGet(LocalSlotRef),
    LocalSet(LocalSlotRef, Box<Node>),

    Block(Vec<Node>),
    Call(FunctionRef, Vec<Node>)
}

pub struct FunctionRef {}

pub struct Function {
    pub param_types: Vec<Type>,
    pub captures: Vec<Capture>,
    pub local_types: Vec<Type>,
    pub body: Node,
    pub return_type: Type
}

pub enum Type {
    Ptr,
    Int,
    Bool,
    Float
}

pub enum Any {
    Int(i64),
    Bool(bool),
    Float(f64),
    String(Box<str>),
    // StructInstanceRef(StructInstanceRef),
    // InterfaceInstanceRef(InterfaceInstanceRef)
}
// pub struct StructInstanceRef {}
// pub struct InterfaceInstanceRef {}
