use crate::compiler::lexical_scope::{Capture, GlobalSlotRef, LocalSlotRef};
use crate::frontend::Location;

pub struct MIR {
    pub node: Node,
    pub typ: CompileTimeValueRef,
    pub location: Location
}

pub enum Node {
    CompileTimeRef(CompileTimeValueRef),

    GlobalRef(GlobalSlotRef),

    ConstStringRef(usize),

    LiteralI32(i32),
    LiteralI64(i64),
    LiteralF32(f32),
    LiteralF64(f64),

    LocalSet(LocalSlotRef, Box<MIR>),
    LocalGet(LocalSlotRef),

    Block(Vec<MIR>),

    Call(FunctionRef, Vec<MIR>)
}

#[derive(Copy)]
pub struct CompileTimeValueRef {
    i: usize
}

pub struct FunctionTemplate {
    pub body: MIR
}

pub struct Function {
    pub param_types: Vec<CompileTimeValueRef>,
    pub captures: Vec<Capture>,
    pub body: MIR,
    pub return_type: CompileTimeValueRef
}

#[derive(Copy)]
pub struct FunctionRef {
    i: usize
}