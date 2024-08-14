use crate::ast;
use crate::ir::Value;

#[derive(Debug)]
pub struct Module {
    pub functions: Vec<Function>,
    pub main: Function
}

#[derive(Debug)]
pub struct Function {
    pub captures: Vec<Capture>,
    pub params: Vec<Param>,
    pub return_type: Option<IR>,
    pub body: IR
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Capture {
    pub from: CaptureFrom
}

#[derive(Debug)]
pub struct Param {
    pub param_ref: ParamRef
}

#[derive(Debug)]
pub struct IR {
    pub node: Node,
    pub location: ast::Location
}

#[derive(Debug)]
pub enum Node {
    Nop,

    Constant(Value),

    GlobalRef(GlobalRef),
    ParamRef(ParamRef),
    LocalRef(LocalRef),

    LocalSet(LocalRef, Box<IR>),

    Block(Vec<IR>),

    Call(Box<str>, Box<IR>, Vec<IR>),
    CreateClosure(FunctionRef, Vec<CaptureFrom>),

    If(Box<IR>, Box<IR>, Option<Box<IR>>)
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct FunctionRef { pub i: usize }

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct GlobalRef { pub i: usize, pub comptime: bool }

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct CaptureRef { pub i: usize, pub comptime: bool }

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ParamRef { pub i: usize, pub comptime: bool }

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct LocalRef { pub i: usize, pub comptime: bool }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureFrom {
    Capture(CaptureRef),
    Param(ParamRef),
    Local(LocalRef)
}