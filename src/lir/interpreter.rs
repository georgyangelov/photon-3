use crate::ir::{Globals, Value};
use crate::lir;
use crate::lir::{BasicBlock, Instruction, ValueRef};

pub struct Interpreter<'a> {
    pub globals: &'a Globals,
    pub functions: &'a [lir::Function]
}

// TODO: Optimize into a single stack frame array for locality
pub struct StackFrame {
    pub params: Vec<Value>,
    pub locals: Vec<Value>,
    pub captures: Vec<Value>
}

impl <'a> Interpreter<'a> {
    pub fn eval_call(&self, func: lir::Function, params: Vec<Value>, captures: Vec<Value>) -> Value {
        let mut locals = Vec::new();
        locals.resize(func.local_count, Value::None);

        let mut frame = StackFrame { params, locals, captures };

        self.eval(&mut frame, &func.body)
    }

    pub fn eval(&self, frame: &mut StackFrame, block: &BasicBlock) -> Value {
        for instruction in &block.code {
            match instruction {
                Instruction::LocalSet(local_ref, value_ref, _) => {
                    let value = self.resolve(frame, *value_ref);

                    frame.locals[local_ref.i] = value;
                }
                Instruction::CallIntrinsic(local_ref, func, args) => todo!("Support intrinsic calls"),
                Instruction::Return(value_ref) => return self.resolve(frame, *value_ref),
                Instruction::If(_, _, _, _, _) => todo!("Support if")
            }
        }

        Value::None
    }

    #[inline]
    fn resolve(&self, frame: &StackFrame, value_ref: ValueRef) -> Value {
        match value_ref {
            ValueRef::None => Value::None,
            ValueRef::Bool(value) => Value::Bool(value),
            ValueRef::Int(value) => Value::Int(value),
            ValueRef::Float(value) => Value::Float(value),
            ValueRef::Param(param_ref) => frame.params[param_ref.i].clone(),
            ValueRef::Local(local_ref) => frame.locals[local_ref.i].clone()
        }
    }
}