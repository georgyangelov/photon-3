use crate::ir;
use crate::ir::Type;

#[derive(Debug)]
pub struct Module {
    pub functions: Vec<Function>,
    pub main: Function
}

#[derive(Debug)]
pub struct Function {
    pub signature: FunctionSignature,
    pub local_count: usize,
    pub body: BasicBlock
}

#[derive(Debug)]
pub struct BasicBlock {
    pub code: Vec<Instruction>
}

#[derive(Debug)]
pub enum Instruction {
    LocalSet(LocalRef, ValueRef, Type),

    CallIntrinsic(LocalRef, IntrinsicFn, Vec<ValueRef>),

    CreateStaticClosure(LocalRef, Vec<ValueRef>),
    CallStaticClosure(LocalRef, FunctionRef, ValueRef, Vec<ValueRef>),

    Return(ValueRef),

    If(LocalRef, ValueRef, BasicBlock, BasicBlock, Type)
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
pub struct FunctionRef { pub i: usize }

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CaptureRef { pub i: usize }

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ParamRef { pub i: usize }

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct LocalRef { pub i: usize }

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub captures: Vec<Type>,
    pub returns: Type
}

#[derive(Debug, Clone, Copy)]
pub enum IntrinsicFn {
    AddInt
}

// PERFORMANCE: Optimize to not create new objects every time
impl IntrinsicFn {
    pub fn signature(&self, _arg_types: &[Type]) -> FunctionSignature {
        match self {
            IntrinsicFn::AddInt => FunctionSignature {
                params: vec![Type::Int, Type::Int],
                captures: Vec::new(),
                returns: Type::Int
            }
        }
    }
}
