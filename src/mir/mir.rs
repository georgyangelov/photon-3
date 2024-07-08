use crate::mir::lexical_scope::*;
use crate::ast;

#[derive(Debug)]
pub struct Module {
    pub functions: Vec<Function>,

    pub comptime_export_count: usize,
    pub comptime_main: Function,

    pub runtime_main: Function
}

#[derive(Debug)]
pub struct Function {
    pub param_count: usize,
    pub param_types: Vec<Option<ComptimeExportRef>>,
    pub return_type: Option<ComptimeExportRef>,

    pub local_count: usize,
    pub captures: Vec<Capture>,
    pub body: MIR,
}

#[derive(Debug)]
pub struct MIR {
    pub node: Node,
    pub location: ast::Location
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

    LocalGet(StackFrameLocalRef),
    LocalSet(StackFrameLocalRef, Box<MIR>),

    Block(Vec<MIR>),

    Call(Box<str>, Box<MIR>, Vec<MIR>),
    CreateClosure(FunctionRef, Vec<CaptureFrom>),

    If(Box<MIR>, Box<MIR>, Option<Box<MIR>>),
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct FunctionRef { pub i: usize }

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct GlobalRef { pub i: usize }

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ComptimeExportRef { pub i: usize }

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct StackFrameLocalRef { pub i: usize }

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ParamRef { pub i: usize }

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct CaptureRef { pub i: usize }