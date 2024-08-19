use crate::vec_map::VecMap;
use crate::ast;
use crate::ir::{IntrinsicFn, Type, Value};

#[derive(Debug)]
pub struct PreComptimeModule {
    pub functions: Vec<TFunction>,
    pub main: TFunction
}

#[derive(Debug)]
pub struct PostComptimeModule {
    // TODO: Make sure that these are only functions used at runtime
    pub functions: Vec<RFunction>,
    pub main: RFunction
}

#[derive(Debug)]
pub struct TFunction {
    pub captures: VecMap<CaptureRef, TCapture>,
    pub params: VecMap<ParamRef, TParam>,
    pub locals: VecMap<LocalRef, TLocal>,
    pub return_type: Option<IR>,
    pub body: IR
}

#[derive(Debug)]
pub struct RFunction {
    pub captures: VecMap<CaptureRef, RCapture>,
    pub params: VecMap<ParamRef, RParam>,
    pub locals: VecMap<LocalRef, RLocal>,
    pub return_type: Type,
    pub body: IR
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TCapture {
    pub from: CaptureFrom,
    pub comptime: bool
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RCapture {
    pub from: CaptureFrom,
    pub typ: Type
}

#[derive(Debug)]
pub struct TParam {
    pub typ: Option<Box<IR>>,
    pub comptime: bool
}

#[derive(Debug)]
pub struct RParam {
    pub typ: Type
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TLocal {
    pub comptime: bool
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RLocal {
    pub typ: Type
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

    DynamicCall(Box<str>, Box<IR>, Vec<IR>),
    DynamicCreateClosure(FunctionTemplateRef, VecMap<CaptureRef, CaptureFrom>),

    StaticCallIntrinsic(IntrinsicFn, Vec<IR>),
    StaticCall(FunctionRef, Vec<IR>),
    // StaticCreateClosure(FunctionRef, Vec<CaptureFrom>),

    If(Box<IR>, Box<IR>, Option<Box<IR>>)
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct FunctionTemplateRef { pub i: usize }

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