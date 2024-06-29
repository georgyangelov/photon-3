use crate::compiler::lexical_scope::*;
use crate::frontend::Location;

#[derive(Debug)]
pub struct Module {
    // TODO: Optimize away functions that are used only from comptime
    pub functions: Vec<Function>,

    pub comptime_export_count: usize,
    pub comptime_main: Function,

    pub runtime_main: Function
}

#[derive(Debug)]
pub struct Function {
    // pub param_types: Vec<ComptimeExportRef>,
    pub frame_layout: FrameLayout,
    pub param_count: usize,
    pub local_count: usize,
    pub captures: Vec<Capture>,
    pub body: MIR,
    // pub return_type: ComptimeExportRef
}

#[derive(Debug)]
pub struct MIR {
    pub node: Node,
    // pub typ: ComptimeExportRef,
    pub location: Location
}

#[derive(Debug)]
pub enum Node {
    Nop,

    CompileTimeGet(ComptimeExportRef),
    CompileTimeSet(ComptimeExportRef, Box<MIR>),

    GlobalRef(GlobalRef),

    ConstStringRef(usize),

    LiteralBool(bool),
    // LiteralI32(i32),
    LiteralI64(i64),
    // LiteralF32(f32),
    LiteralF64(f64),

    ParamRef(ParamRef),
    CaptureRef(CaptureRef),

    LocalSet(StackFrameLocalRef, Box<MIR>),
    LocalGet(StackFrameLocalRef),

    Block(Vec<MIR>),

    Call(Box<str>, Box<MIR>, Vec<MIR>),
    CreateClosure(FunctionRef, Vec<CaptureFrom>),

    If(Box<MIR>, Box<MIR>, Option<Box<MIR>>),
}

#[derive(Debug)]
pub struct FrameLayout {
    pub size: usize
}

#[derive(Copy, Clone, Debug)]
pub struct FunctionRef {
    pub i: usize
}