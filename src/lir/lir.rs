use crate::lir::Value;
use crate::mir;
use crate::types::{FunctionSignature, IntrinsicFn, Type};

pub struct Module {
    pub constants: Vec<Value>,

    /// None here means that the function is not used at runtime so can be omitted
    pub functions: Vec<Option<Function>>,

    pub main: Function,
}

#[derive(Debug)]
pub struct Function {
    pub capture_types: Vec<Type>,
    pub param_types: Vec<Type>,
    pub return_type: Type,
    pub local_types: Vec<Type>,
    pub entry: BasicBlock
}

#[derive(Debug)]
pub struct BasicBlock {
    pub code: Vec<Instruction>
}

#[derive(Debug)]
pub enum Instruction {
    // ParamGet(ParamRef, Type),
    // LocalGet(LocalRef, Type),

    LocalSet(LocalRef, ValueRef, Type),
    CompileTimeSet(mir::ComptimeExportRef, ValueRef, Type),

    // TODO: Type conversion operators
    // TODO: Type assertion

    // TODO: Make this a regular struct (once we have types for them) and remove the special-cased
    //       closure-related calls here and below
    /// Creating a closure struct
    CreateClosure(LocalRef, FunctionRef, Vec<ValueRef>),

    /// A call to a function based on the function's name
    CallDynamicFunction(LocalRef, String, Vec<ValueRef>),

    /// A call to a built-in function (compiler intrinsic)
    CallIntrinsicFunction(LocalRef, IntrinsicFn, Vec<ValueRef>, FunctionSignature),

    /// A call to a non-closure function known at compile time
    CallStaticFunction(LocalRef, FunctionRef, Vec<ValueRef>, FunctionSignature),

    /// A call to a closure function known at compile time
    CallStaticClosureFunction(LocalRef, FunctionRef, ValueRef, Vec<ValueRef>, FunctionSignature),

    /// A call to a non-closure function not known at compile-time (through a pointer)
    CallPtrFunction(LocalRef, ValueRef, Vec<ValueRef>, FunctionSignature),

    /// A call to a closure function not known at compile-time (through a pointer)
    CallPtrClosureFunction(LocalRef, ValueRef, ValueRef, Vec<ValueRef>, FunctionSignature),

    Return(ValueRef, Type),

    If(ValueRef, BasicBlock, BasicBlock, Type)
}

#[derive(Debug, Clone, Copy)]
pub enum ValueRef {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Global(mir::GlobalRef),
    ComptimeExport(mir::ComptimeExportRef),
    Const(ConstRef),
    Capture(CaptureRef),
    Param(ParamRef),
    Local(LocalRef)
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ConstRef { pub i: usize }

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CaptureRef { pub i: usize }

#[derive(Debug, Clone, Copy)]
pub struct ParamRef { pub i: usize }

#[derive(Debug, Clone, Copy)]
pub struct LocalRef { pub i: usize }

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FunctionRef { pub i: usize }