use crate::ir;

#[derive(Debug)]
pub struct Module {
    pub functions: Vec<Function>,
    pub main: Function
}

#[derive(Debug)]
pub struct Function {
    pub capture_types: Vec<ir::Type>,
    pub param_types: Vec<ir::Type>,
    pub return_type: ir::Type,
    pub local_count: usize,
    pub body: BasicBlock
}

#[derive(Debug)]
pub struct BasicBlock {
    pub code: Vec<Instruction>
}

#[derive(Debug)]
pub enum Instruction {
    LocalSet(LocalRef, ValueRef, ir::Type),

    CallIntrinsic(LocalRef, ir::IntrinsicFn, Vec<ValueRef>),
    // Call(FunctionRef, Vec<IR>),
    // CreateClosure(FunctionRef, Vec<CaptureFrom>),

    Return(ValueRef),

    If(LocalRef, ValueRef, BasicBlock, BasicBlock, ir::Type)
}

#[derive(Debug, Clone, Copy)]
pub enum ValueRef {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    // Global(GlobalRef),
    // Const(ConstRef),
    // Capture(CaptureRef),
    Param(ParamRef),
    Local(LocalRef)
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ParamRef { pub i: usize }

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct LocalRef { pub i: usize }