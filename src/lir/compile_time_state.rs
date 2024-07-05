use std::collections::HashSet;
use std::rc::Rc;
use crate::lir::compile_time_state::ResolvedFn::Intrinsic;
use crate::lir::{Function, FunctionRef, Value};
use crate::mir;
use crate::types::{IntrinsicFn, Type};
use crate::types::IntrinsicFn::AddInt;

pub struct CompileTimeState {
    pub comptime_exports: Vec<Value>,
    pub functions: Vec<CompilingFunction>,
    pub runtime_used_functions: HashSet<FunctionRef>

    // struct_types: Arena<Type>,
    // interface_types: Arena<Type>
}

impl CompileTimeState {
    pub fn new(export_count: usize) -> Self {
        let mut comptime_exports = Vec::new();

        // TODO: Make sure exports are not used before being defined
        comptime_exports.resize(export_count, Value::None);

        Self {
            comptime_exports,
            functions: Vec::new(),
            runtime_used_functions: HashSet::new()
        }
    }

    pub fn resolve_fn(&self, name: &str, arg_types: &[Type]) -> Option<ResolvedFn> {
        match (arg_types[0], name) {
            (Type::Int, "+") => Some(Intrinsic(AddInt)),
            (Type::Closure(_), "call") => todo!("Support dynamic 'call' on Closure values in the interpreter"),
            _ => None
        }
    }

    pub fn mark_as_used_at_runtime(&mut self, func_ref: FunctionRef) {
        self.runtime_used_functions.insert(func_ref);
    }

    pub fn get_compiled_fn(&self, func_ref: FunctionRef) -> Rc<Function> {
        match &self.functions[func_ref.i] {
            CompilingFunction::Pending => panic!("Tried to get function while it's still compiling"),
            CompilingFunction::Compiled(func) => func.clone()
        }
    }
}

#[derive(Clone, Copy)]
pub enum ResolvedFn {
    Intrinsic(IntrinsicFn),
    Function(mir::FunctionRef)
}

pub enum CompilingFunction {
    Pending,
    Compiled(Rc<Function>)
}