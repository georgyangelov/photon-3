use std::collections::HashMap;
use crate::ir;
use crate::ir::{Globals, Type, Value};

pub struct Interpreter<'a> {
    globals: &'a Globals,
    functions: Vec<ir::Function>
}

struct ComptimeStackFrame {
    arg_types: Vec<Type>,
    args: Vec<Option<Value>>,

    capture_types: Vec<Type>,
    captures: Vec<Option<Value>>,

    local_types: Vec<Option<Type>>,
    locals: Vec<Option<Value>>,

    return_type: Option<Type>
}

impl <'a> Interpreter<'a> {
    pub fn eval_comptime(globals: &'a Globals, module: ir::PreComptimeModule) -> ir::PostComptimeModule {
        let mut interpreter = Self {
            globals,
            functions: Vec::new()
        };
        let main = interpreter.specialize_function(
            module.main,
            vec![],
            vec![],
            vec![],
            vec![]
        );

        ir::PostComptimeModule {
            functions: interpreter.functions,
            main
        }
    }

    fn specialize_function(
        &mut self,
        // TODO: Pass a closure here instead of raw FunctionTemplate, we need the types for the
        //       specialization
        func: ir::FunctionTemplate,
        arg_types: Vec<Type>,
        args: Vec<Option<Value>>,
        capture_types: Vec<Type>,
        captures: Vec<Option<Value>>
    ) -> ir::Function {
        // TODO: If the function body resolves to a constant - return that constant directly
        //       instead of wrapping in a function?

        let mut locals = Vec::new();
        locals.resize(func.locals_comptime.len(), None);

        let mut local_types = Vec::new();
        local_types.resize(locals.len(), None);

        let mut stack_frame = ComptimeStackFrame {
            arg_types,
            args,

            capture_types,
            captures,

            local_types,
            locals,

            return_type: None
        };

        // TODO: Eval & type-check arguments

        let return_type = match &func.return_type {
            None => None,
            Some(ir) => {
                let (value, _) = self.eval_comptime_ir(&mut stack_frame, ir);

                Some(match value {
                    Value::Type(typ) => typ,
                    value => panic!("The return value expression must evaluate to a type, got {:?}", value)
                })
            }
        };

        let (runtime_body, body_typ) = self.eval_ir(&mut stack_frame, &func.body, false);

        let return_type = match return_type {
            None => body_typ,
            Some(typ) => {
                todo!("Compare return_type with body_type")
            }
        };

        let mut runtime_captures = HashMap::new();
        for (i, capture) in func.captures.iter().enumerate() {
            if !capture.comptime {
                runtime_captures.insert(
                    ir::CaptureRef { i, comptime: false },
                    ir::RuntimeCapture { from: capture.from, typ: stack_frame.capture_types[i] }
                );
            }
        }

        let mut runtime_locals = HashMap::new();
        for (i, local_comptime) in func.locals_comptime.iter().enumerate() {
            if !local_comptime {
                runtime_locals.insert(
                    ir::LocalRef { i, comptime: false },
                    stack_frame.local_types[i].expect("Missing type information for local")
                );
            }
        }

        let mut runtime_arg_types = HashMap::new();
        for (i, param) in func.params.iter().enumerate() {
            if !param.comptime {
                runtime_arg_types.insert(
                    ir::ParamRef { i, comptime: false },
                    stack_frame.arg_types[i]
                );
            }
        }

        ir::Function {
            captures: runtime_captures,
            locals: runtime_locals,
            param_types: runtime_arg_types,
            return_type,
            body: runtime_body
        }
    }

    fn eval_comptime_ir(&mut self, frame: &mut ComptimeStackFrame, ir: &ir::IR) -> (Value, Type) {
        let (body, typ) = self.eval_ir(frame, ir, true);

        let value = match body.node {
            ir::Node::Nop => Value::None,
            ir::Node::Constant(value) => value,
            result => panic!("Could not fully eval IR {:?}, got {:?}", ir, result)
        };

        if typ == Type::Any {
            todo!("Support specializing Any types");
        }

        (value, typ)
    }

    fn eval_ir(
        &mut self,
        frame: &mut ComptimeStackFrame,
        ir: &ir::IR,
        in_comptime: bool
    ) -> (ir::IR, Type) {
        let location = ir.location.clone();
        let (node, typ) = match &ir.node {
            ir::Node::Nop => (ir::Node::Nop, Type::None),
            ir::Node::Constant(value) => (ir::Node::Constant(value.clone()), value.type_of()),
            ir::Node::GlobalRef(global_ref) => {
                let ir = if global_ref.comptime {
                    todo!("Resolve global value as constant");
                    todo!("Support specializing Any types");
                } else {
                    ir::Node::GlobalRef(*global_ref)
                };

                (ir, self.globals.globals[global_ref.i].value.type_of())
            }
            ir::Node::ParamRef(param_ref) => {
                let ir = if param_ref.comptime {
                    todo!("Resolve param value as constant");
                    todo!("Support specializing Any types");
                } else {
                    ir::Node::ParamRef(*param_ref)
                };

                (ir, frame.arg_types[param_ref.i])
            }
            ir::Node::LocalRef(local_ref) => {
                let ir = if local_ref.comptime {
                    todo!("Resolve local value as constant");
                    todo!("Support specializing Any types");
                } else {
                    ir::Node::LocalRef(*local_ref)
                };

                (ir, frame.local_types[local_ref.i].expect("Used param before definition"))
            }
            ir::Node::CaptureRef(capture_ref) => {
                let ir = if capture_ref.comptime {
                    todo!("Resolve captured value as constant");
                    todo!("Support specializing Any types");
                } else {
                    ir::Node::CaptureRef(*capture_ref)
                };

                (ir, frame.capture_types[capture_ref.i])
            }
            ir::Node::LocalSet(local_ref, value_ir) => {
                let (value, typ) = self.eval_ir(frame, value_ir, local_ref.comptime);

                let ir = if local_ref.comptime {
                    todo!("Resolve local_set during comptime");
                    todo!("Support specializing Any types");
                } else {
                    ir::Node::LocalSet(*local_ref, Box::new(value))
                };

                frame.local_types[local_ref.i] = Some(typ);

                (ir, typ)
            }
            ir::Node::Block(irs) => {
                if irs.len() == 0 {
                    (ir::Node::Constant(Value::None), Type::None)
                } else {
                    let mut results = Vec::with_capacity(irs.len());
                    let mut result_type = Type::None;
                    for ir in irs {
                        let (ir, typ) = self.eval_ir(frame, ir, in_comptime);
                        result_type = typ;

                        // TODO: Remove if it resolves to a constant and it's not the last expression
                        results.push(ir);
                    }

                    (ir::Node::Block(results), result_type)
                }
            }
            ir::Node::Comptime(ir) => {
                let (result_ir, result_typ) = self.eval_comptime_ir(frame, ir);

                // Doing this because of the location & to make sure we checked that it's fully
                // evaluated
                (ir::Node::Constant(result_ir), result_typ)
            },
            ir::Node::Call(name, target, args) => {
                todo!("Eval argument types, lookup function based on target name, specialize it")
            }
            ir::Node::CreateClosure(func_ref, captures) => {
                todo!("Support CreateClosure in the interpreter")
            }
            ir::Node::If(_, _, _) => {
                todo!("Support 'if' in the interpreter")
            }
        };

        (ir::IR { node, location }, typ)
    }
}