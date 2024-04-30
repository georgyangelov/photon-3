use crate::compiler::lexical_scope::{Capture, CompileTimeSlotRef, GlobalSlotRef, LocalSlotRef};
use crate::frontend::Location;

pub struct MIR {
    pub node: Node,
    pub typ: CompileTimeSlotRef,
    pub location: Location
}

pub enum Node {
    CompileTimeRef(CompileTimeSlotRef),
    CompileTimeSet(CompileTimeSlotRef, Box<MIR>),

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

pub struct FunctionTemplate {
    pub body: MIR
}

pub struct Function {
    pub param_types: Vec<CompileTimeSlotRef>,
    pub captures: Vec<Capture>,
    pub body: MIR,
    pub return_type: CompileTimeSlotRef
}

#[derive(Copy, Clone)]
pub struct FunctionRef {
    i: usize
}