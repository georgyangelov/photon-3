use std::rc::Rc;
use crate::mir;
use crate::lir::*;
use crate::mir::lexical_scope::CaptureFrom;

pub struct MIRInterpreter {
    stack: Vec<StackFrame>,
    exports: Vec<Value>
}

struct StackFrame {
    captures: Vec<Value>,
    args: Vec<Value>,
    locals: Vec<Value>
}

pub struct ComptimeEvalResult {
    pub exports: Vec<Value>
}

impl MIRInterpreter {
    // TODO: Actual error handling instead of panics
    pub fn eval_comptime(mir_module: &mir::Module) -> ComptimeEvalResult {
        let stack = Vec::new();
        let exports = empty_vec(mir_module.comptime_export_count);

        let mut interpreter = MIRInterpreter { stack, exports };

        interpreter.call_function(&mir_module, &mir_module.comptime_main, vec![], vec![]);

        ComptimeEvalResult { exports: interpreter.exports }
    }

    fn call_function(
        &mut self,
        module: &mir::Module,
        func: &mir::Function,
        captures: Vec<Value>,
        args: Vec<Value>
    ) -> Value {
        self.stack.push(StackFrame {
            locals: empty_vec(func.local_count),
            args,
            captures
        });

        let result = self.eval(module, &func.body);

        self.stack.pop();

        result
    }

    fn eval(&mut self, module: &mir::Module, mir: &mir::MIR) -> Value {
        match &mir.node {
            mir::Node::Nop => Value::None,

            mir::Node::CompileTimeGet(export_ref) => self.exports[export_ref.i].clone(),
            mir::Node::CompileTimeSet(export_ref, mir) => {
                let value = self.eval(module, mir);

                self.exports[export_ref.i] = value;

                Value::None
            }

            mir::Node::GlobalRef(_) => todo!("Support GlobalRef"),

            mir::Node::ConstStringRef(_) => todo!("Support ConstStringRef"),

            mir::Node::LiteralBool(value) => Value::Bool(*value),
            mir::Node::LiteralI64(value) => Value::Int(*value),
            mir::Node::LiteralF64(value) => Value::Float(*value),

            mir::Node::ParamRef(param_ref) => self.current_frame().args[param_ref.i].clone(),
            mir::Node::CaptureRef(capture_ref) => self.current_frame().captures[capture_ref.i].clone(),

            mir::Node::LocalGet(local_ref) => self.current_frame().locals[local_ref.i].clone(),
            mir::Node::LocalSet(local_ref, mir) => {
                let value = self.eval(module, mir);

                self.current_frame().locals[local_ref.i] = value;

                Value::None
            }

            mir::Node::Block(mirs) => {
                let mut result = Value::None;

                for mir in mirs {
                    result = self.eval(module, mir);
                }

                result
            }

            mir::Node::Call(name, target, args) => {
                let target_value = self.eval(module, target);
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    let value = self.eval(module, arg);
                    arg_values.push(value);
                }

                self.call(module, &name, target_value, arg_values)
            }

            mir::Node::CreateClosure(func_ref, captures) => {
                let mut capture_values = Vec::with_capacity(captures.len());
                for capture in captures {
                    let frame = self.current_frame();

                    capture_values.push(match capture {
                        CaptureFrom::Capture(capture_ref) => frame.captures[capture_ref.i].clone(),
                        CaptureFrom::Param(param_ref) => frame.args[param_ref.i].clone(),
                        CaptureFrom::Local(local_ref) => frame.locals[local_ref.i].clone()
                    });
                }

                let captures = Rc::new(capture_values);

                Value::Closure(*func_ref, captures)
            }

            mir::Node::If(_, _, _) => todo!("Support if")
        }
    }

    fn call(&mut self, module: &mir::Module, name: &str, target: Value, args: Vec<Value>) -> Value {
        println!("name: {:?}", name);
        println!("args: {:?}", args);

        if name == "+" {
            Value::Int(target.assert_int() + args[0].assert_int())
        } else if name == "call" {
            let (func_ref, captures) = target.assert_closure();

            self.call_function(module, &module.functions[func_ref.i], captures.clone(), args)
        } else {
            panic!("Unknown function {}", name)
        }
    }

    fn current_frame(&mut self) -> &mut StackFrame {
        let i = self.stack.len() - 1;

        &mut self.stack[i]
    }
}

fn empty_vec(size: usize) -> Vec<Value> {
    let mut result = Vec::with_capacity(size);
    result.resize(size, Value::None);
    result
}