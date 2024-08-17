use std::collections::{HashMap, HashSet};
use crate::ast;
use crate::ir::{Type, Value};

#[derive(Debug)]
pub struct PreComptimeModule {
    pub functions: Vec<FunctionTemplate>,
    pub main: FunctionTemplate
}

#[derive(Debug)]
pub struct PostComptimeModule {
    pub functions: Vec<Function>,
    pub main: Function
}

#[derive(Debug)]
pub struct FunctionTemplate {
    pub captures: Vec<CaptureTemplate>,
    pub params: Vec<Param>,
    pub return_type: Option<IR>,
    pub locals_comptime: Vec<bool>,
    pub body: IR
}

#[derive(Debug)]
pub struct Function {
    // TODO: Replace with something more performant for small arrays
    pub captures: HashMap<CaptureRef, RuntimeCapture>,
    pub locals: HashMap<LocalRef, Type>,
    pub param_types: HashMap<ParamRef, Type>,
    pub return_type: Type,
    pub body: IR
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaptureTemplate {
    pub from: CaptureFrom,
    pub comptime: bool
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeCapture {
    pub from: CaptureFrom,
    pub typ: Type
}

#[derive(Debug)]
pub struct Param {
    // TODO: Do we need this?
    // pub param_ref: ParamRef,
    pub typ: Option<Box<IR>>,
    pub comptime: bool
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
    CaptureRef(CaptureRef),

    LocalSet(LocalRef, Box<IR>),

    Block(Vec<IR>),
    Comptime(Box<IR>),

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