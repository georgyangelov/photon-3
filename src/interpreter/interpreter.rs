use crate::compiler::mir::{FrameLayout, Function, Node};
use crate::compiler::{mir, Module};
use crate::interpreter::interpreter::FunctionToCall::{Photon, Rust};
use crate::interpreter::Value;

pub struct Interpreter {
    stack: Vec<Value>,
    stack_offset: usize
}

impl Interpreter {
    pub fn new() -> Self {
        let mut stack = Vec::new();
        stack.resize(10 * 1024, Value::None);

        Self {
            stack,
            stack_offset: 0
        }
    }

    pub fn eval_module(&mut self, module: Module) -> Value {
        let main = module.runtime_main;

        self.push_stack_for_call(0, &main, vec![]);
        let result = self.eval_mir(&main.body);
        self.pop_stack_after_call(0);

        result
    }

    fn eval_mir(&mut self, mir: &mir::MIR) -> Value {
        match &mir.node {
            Node::Nop => Value::None,

            Node::CompileTimeRef(_) => todo!("Implement CompileTimeRef eval"),
            Node::CompileTimeSet(_, _) => todo!("Implement CompileTimeSet eval"),
            Node::GlobalRef(_) => todo!("Implement GlobalRef eval"),
            Node::ConstStringRef(_) => todo!("Implement ConstStringRef eval"),

            Node::LiteralI8(value) => Value::I8(*value),
            Node::LiteralI64(value) => Value::I64(*value),
            Node::LiteralF64(value) => Value::F64(*value),

            Node::LocalSet(local_ref, mir) => {
                self.stack[self.stack_offset + local_ref.i] = self.eval_mir(mir);

                Value::None
            },
            Node::LocalGet(local_ref) => self.stack[self.stack_offset + local_ref.i].clone(),

            Node::Block(mirs) => {
                let mut result = Value::None;

                for mir in mirs {
                    result = self.eval_mir(mir);
                }

                result
            },

            Node::Call(name, target, args) => {
                let target = self.eval_mir(target);
                let func = self.find_func(&target, name);

                let mut arg_values = Vec::with_capacity(args.len() + 1);
                arg_values.push(target);

                for arg in args {
                    let value = self.eval_mir(arg);
                    arg_values.push(value);
                }

                match func {
                    None => todo!("Error handling - could not find function"),
                    Some(Rust(rust_fn)) => rust_fn(arg_values),
                    Some(Photon(photon_fn)) => todo!("Call photon function"),
                }
            },
        }
    }

    fn find_func(&self, target: &Value, name: &str) -> Option<FunctionToCall> {
        match target {
            Value::None => None,
            Value::I8(_) => None,
            Value::I64(_) => {
                match name {
                    "+" => Some(Rust(add_i64)),
                    _ => None
                }
            }
            Value::F64(_) => None
        }
    }

    #[inline]
    fn push_stack_for_call(
        &mut self,
        current_frame_size: usize,
        target_func: &Function,
        args: Vec<Value>,
    ) {
        if self.stack_offset + current_frame_size + target_func.frame_layout.size >= self.stack.len() {
            panic!("Stack overflow");
        }

        for (i, arg) in args.into_iter().enumerate() {
            self.stack[self.stack_offset + current_frame_size + i] = arg;
        }

        for capture in &target_func.captures {
            self.stack[self.stack_offset + current_frame_size + capture.to.i] =
                self.stack[self.stack_offset + capture.from.i].clone();
        }

        self.stack_offset += current_frame_size
    }

    #[inline]
    fn pop_stack_after_call(&mut self, current_frame_size: usize) {
        self.stack_offset -= current_frame_size
    }
}

enum FunctionToCall {
    Photon(Function),
    Rust(fn(Vec<Value>) -> Value)
}

fn add_i64(args: Vec<Value>) -> Value {
    let [a, b] = args.try_into().expect("Invalid args");

    Value::I64(a.expect_i64() + b.expect_i64())
}

// fn expect_arity(args: &Vec<Value>, arity: usize) {
//     if args.len() != arity {
//         todo!("Error handling")
//     }
// }