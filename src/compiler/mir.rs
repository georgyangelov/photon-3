use crate::compiler::lexical_scope::{Capture, ComptimeExportRef, GlobalRef, StackFrameLocalRef};
use crate::frontend::Location;

pub struct MIR {
    pub node: Node,
    // pub typ: ComptimeExportRef,
    pub location: Location
}

pub enum Node {
    Nop,

    CompileTimeRef(ComptimeExportRef),
    CompileTimeSet(ComptimeExportRef, Box<MIR>),

    GlobalRef(GlobalRef),

    ConstStringRef(usize),

    LiteralI8(i8),
    // LiteralI32(i32),
    LiteralI64(i64),
    // LiteralF32(f32),
    LiteralF64(f64),

    LocalSet(StackFrameLocalRef, Box<MIR>),
    LocalGet(StackFrameLocalRef),

    Block(Vec<MIR>),

    Call(Box<str>, Box<MIR>, Vec<MIR>)
}

pub struct FunctionTemplate {
    pub body: MIR
}

pub struct Function {
    // pub param_types: Vec<ComptimeExportRef>,
    pub frame_layout: FrameLayout,
    pub captures: Vec<Capture>,
    pub body: MIR,
    // pub return_type: ComptimeExportRef
}

pub struct FrameLayout {
    pub size: usize
}

#[derive(Copy, Clone)]
pub struct FunctionRef {
    i: usize
}