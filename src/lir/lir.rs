use crate::lir::Value;
use crate::types::{IntrinsicFn, Type};

pub struct Module {
    pub constants: Vec<Value>,
    pub functions: Vec<Function>,
    pub main: Function,
}

pub struct Function {
    pub param_types: Vec<Type>,
    pub return_type: Type,
    pub local_types: Vec<Type>,
    pub entry: BasicBlock
}

pub struct BasicBlock {
    pub code: Vec<Instruction>
}

pub enum Instruction {
    ParamGet(ParamRef, Type),
    LocalGet(LocalRef, Type),

    LocalSet(LocalRef, ValueRef, Type),

    // TODO: Type conversion operators
    // TODO: Type assertion

    CallIntrinsicFunction(LocalRef, IntrinsicFn, Vec<ValueRef>, Type),
    // CallStaticFunction(LocalRef, FunctionRef, Vec<ValueRef>, Type),
    // CallDynamicFunction(LocalRef, ValueRef, Vec<ValueRef>, Type),
    // CallClosureFunction(LocalRef, ValueRef, Vec<ValueRef>, Type),

    Return(ValueRef, Type),

    If(ValueRef, BasicBlock, BasicBlock, Type)
}

#[derive(Clone)]
pub enum ValueRef {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Const(ConstRef),
    Param(ParamRef),
    Local(LocalRef)
}

#[derive(Clone, Copy)]
pub struct ConstRef { pub i: usize }

#[derive(Clone, Copy)]
pub struct ParamRef { pub i: usize }

#[derive(Clone, Copy)]
pub struct LocalRef { pub i: usize }

#[derive(Clone, Copy)]
pub struct FunctionRef { pub i: usize }
