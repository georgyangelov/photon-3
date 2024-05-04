use std::rc::Rc;
use crate::compiler::mir::{Function, Node};
use crate::compiler::{mir, Module};
use crate::backend::interpreter::{Closure, Value};
use std::borrow::Borrow;

pub struct Interpreter {
    stack: Vec<Value>,
    stack_offset: usize,
    current_frame_size: usize
}

#[derive(Debug)]
pub struct ModuleEvalResult {
    pub comptime_exports: Vec<Value>,
    pub result: Value
}

impl Interpreter {
    pub fn new() -> Self {
        let mut stack = Vec::new();
        stack.resize(10 * 1024, Value::None);

        Self {
            stack,
            stack_offset: 0,
            current_frame_size: 0
        }
    }

    pub fn eval_module_comptime(&mut self, module: &Module) -> ModuleEvalResult {
        let mut comptime_exports = Vec::new();
        comptime_exports.resize(module.comptime_export_count, Value::None);

        let main = &module.comptime_main;

        let current_frame_size = self.current_frame_size;
        self.push_stack_for_call(current_frame_size, main, vec![], &[]);
        let result = self.eval_mir(module, &mut comptime_exports, &main.body);
        self.pop_stack_after_call(current_frame_size);

        ModuleEvalResult {
            comptime_exports,
            result
        }
    }

    pub fn eval_module_runtime(&mut self, module: &Module, mut comptime_exports: Vec<Value>) -> Value {
        let main = &module.runtime_main;

        let current_frame_size = self.current_frame_size;
        self.push_stack_for_call(current_frame_size, main, vec![], &[]);
        let result = self.eval_mir(module, &mut comptime_exports, &main.body);
        self.pop_stack_after_call(current_frame_size);

        result
    }

    fn eval_mir(&mut self, module: &Module, exports: &mut Vec<Value>, mir: &mir::MIR) -> Value {
        match &mir.node {
            Node::Nop => Value::None,

            Node::CompileTimeRef(export_ref) => exports[export_ref.i].clone(),
            Node::CompileTimeSet(export_ref, mir) => {
                let value = self.eval_mir(module, exports, mir);

                exports[export_ref.i] = value;

                Value::None
            },
            Node::GlobalRef(_) => todo!("Implement GlobalRef eval"),
            Node::ConstStringRef(_) => todo!("Implement ConstStringRef eval"),

            Node::LiteralBool(value) => Value::Bool(*value),
            Node::LiteralI64(value) => Value::I64(*value),
            Node::LiteralF64(value) => Value::F64(*value),

            Node::LocalSet(local_ref, mir) => {
                self.stack[self.stack_offset + local_ref.i] = self.eval_mir(module, exports, mir);

                Value::None
            },
            Node::LocalGet(local_ref) => self.stack[self.stack_offset + local_ref.i].clone(),

            Node::Block(mirs) => {
                let mut result = Value::None;

                for mir in mirs {
                    result = self.eval_mir(module, exports, mir);
                }

                result
            },

            Node::Call(name, target, args) => {
                let target = self.eval_mir(module, exports, target);
                let func = self.find_func(&target, name);

                let mut arg_values = Vec::with_capacity(args.len() + 1);
                arg_values.push(target);

                for arg in args {
                    let value = self.eval_mir(module, exports, arg);
                    arg_values.push(value);
                }

                match func {
                    None => todo!("Error handling - could not find function"),
                    Some(FunctionToCall::Rust(rust_fn)) => rust_fn(arg_values),
                    Some(FunctionToCall::Closure(closure)) => {
                        let func = &module.functions[closure.function_ref.i];
                        let current_frame_size = self.current_frame_size;
                        let closure: &Closure = closure.borrow();

                        // TODO: Make this better
                        arg_values.remove(0);

                        self.push_stack_for_call(current_frame_size, func, arg_values, &closure.values);
                        let result = self.eval_mir(module, exports, &func.body);
                        self.pop_stack_after_call(current_frame_size);

                        result
                    }
                }
            },

            Node::CreateClosure(fn_ref, local_refs) => {
                let mut values = Vec::new();
                for local_ref in local_refs {
                    values.push(self.stack[self.stack_offset + local_ref.i].clone());
                }

                Value::Closure(Rc::new(Closure {
                    values,
                    function_ref: *fn_ref
                }))
            },

            Node::If(condition, on_true, on_false) => {
                let condition_value = self.eval_mir(module, exports, condition);

                let result = if condition_value.expect_bool() {
                    self.eval_mir(module, exports, on_true)
                } else {
                    if let Some(on_false) = on_false {
                        self.eval_mir(module, exports, on_false)
                    } else {
                        Value::None
                    }
                };

                result
            }
        }
    }

    fn find_func(&self, target: &Value, name: &str) -> Option<FunctionToCall> {
        match target {
            Value::None => None,
            Value::Bool(_) => None,
            Value::I64(_) => {
                match name {
                    "+" => Some(FunctionToCall::Rust(add_i64)),
                    _ => None
                }
            }
            Value::F64(_) => None,
            Value::Closure(closure) => {
                if name == "call" {
                    Some(FunctionToCall::Closure(closure.clone()))
                } else { None }
            }
        }
    }

    #[inline]
    fn push_stack_for_call(
        &mut self,
        parent_frame_size: usize,
        target_func: &Function,
        args: Vec<Value>,
        captures: &[Value]
    ) {
        if self.stack_offset + parent_frame_size + target_func.frame_layout.size >= self.stack.len() {
            panic!("Stack overflow");
        }

        for (i, arg) in args.into_iter().enumerate() {
            self.stack[self.stack_offset + parent_frame_size + i] = arg;
        }

        // TODO: This is not correct - we need to capture from the definition scope, not the
        //       call stack
        for (i, capture) in target_func.captures.iter().enumerate() {
            self.stack[self.stack_offset + parent_frame_size + capture.to.i] =
                captures[i].clone();
        }

        self.stack_offset += parent_frame_size;
        self.current_frame_size = target_func.frame_layout.size;
    }

    #[inline]
    fn pop_stack_after_call(&mut self, parent_frame_size: usize) {
        self.stack_offset -= parent_frame_size
    }
}

enum FunctionToCall {
    Closure(Rc<Closure>),
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