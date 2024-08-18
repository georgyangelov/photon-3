use crate::ir;
use crate::ir::{Globals, Type, Value};
use crate::vec_map::VecMap;

pub struct Interpreter<'a> {
    globals: &'a Globals,
    functions: Vec<ir::RFunction>
}

struct ComptimeStackFrame {
    // TODO: Optimization for the lookups of these VecMaps here?
    param_types: VecMap<ir::ParamRef, Type>,
    comptime_param_values: VecMap<ir::ParamRef, Value>,

    capture_types: VecMap<ir::CaptureRef, Type>,
    comptime_capture_values: VecMap<ir::CaptureRef, Value>,

    local_types: VecMap<ir::LocalRef, StackFrameType>,
    comptime_local_values: VecMap<ir::LocalRef, Value>,

    return_type: Option<Type>
}

struct StackFrameType {
    typ: Type,
    comptime: bool
}

impl <'a> Interpreter<'a> {
    pub fn eval_comptime(globals: &'a Globals, module: ir::PreComptimeModule) -> ir::PostComptimeModule {
        let mut interpreter = Self {
            globals,
            functions: Vec::new()
        };
        let main = interpreter.specialize_function(
            module.main,
            VecMap::new(),
            VecMap::new(),
            VecMap::new(),
            VecMap::new()
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
        func: ir::TFunction,

        param_types: VecMap<ir::ParamRef, Type>,
        comptime_param_values: VecMap<ir::ParamRef, Value>,

        capture_types: VecMap<ir::CaptureRef, Type>,
        comptime_capture_values: VecMap<ir::CaptureRef, Value>
    ) -> ir::RFunction {
        // TODO: If the function body resolves to a constant - return that constant directly
        //       instead of wrapping in a function?

        let mut stack_frame = ComptimeStackFrame {
            param_types,
            comptime_param_values,

            capture_types,
            comptime_capture_values,

            local_types: VecMap::new(),
            comptime_local_values: VecMap::new(),

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

        let mut rparams = VecMap::new();
        for (param_ref, param) in func.params.iter() {
            if !param.comptime {
                rparams.insert_push(*param_ref, ir::RParam {
                    // TODO: Yuck
                    typ: *stack_frame.param_types.get(param_ref).unwrap()
                });
            }
        }

        // TODO: Is this correct or do we need to determine those dynamically as we scan the IR?
        let mut rcaptures = VecMap::new();
        for (capture_ref, capture) in func.captures.iter() {
            if !capture.comptime {
                rcaptures.insert_push(*capture_ref, ir::RCapture {
                    from: capture.from,
                    // TODO: Yuck
                    typ: *stack_frame.capture_types.get(capture_ref).unwrap()
                });
            }
        }

        let mut rlocals = VecMap::new();
        for (local_ref, local) in stack_frame.local_types.iter() {
            if !local.comptime {
                rlocals.insert_push(*local_ref, ir::RLocal {
                    typ: local.typ
                });
            }
        }

        ir::RFunction {
            captures: rcaptures,
            locals: rlocals,
            params: rparams,
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

                (ir, *frame.param_types.get(param_ref).expect("Missing param type"))
            }
            ir::Node::LocalRef(local_ref) => {
                let ir = if local_ref.comptime {
                    todo!("Resolve local value as constant");
                    todo!("Support specializing Any types");
                } else {
                    ir::Node::LocalRef(*local_ref)
                };

                (ir, frame.local_types.get(local_ref).expect("Used param before definition").typ)
            }
            ir::Node::CaptureRef(capture_ref) => {
                let ir = if capture_ref.comptime {
                    todo!("Resolve captured value as constant");
                    todo!("Support specializing Any types");
                } else {
                    ir::Node::CaptureRef(*capture_ref)
                };

                (ir, *frame.capture_types.get(capture_ref).expect("Missing capture type"))
            }
            ir::Node::LocalSet(local_ref, value_ir) => {
                let (value, typ) = self.eval_ir(frame, value_ir, local_ref.comptime);

                let ir = if local_ref.comptime {
                    todo!("Resolve local_set during comptime");
                    todo!("Support specializing Any types");
                } else {
                    ir::Node::LocalSet(*local_ref, Box::new(value))
                };

                frame.local_types.insert(*local_ref, StackFrameType {
                    typ,
                    comptime: local_ref.comptime
                });

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
            ir::Node::DynamicCall(name, target, args) => {
                todo!("Eval argument types, lookup function based on target name, specialize it")
            }
            ir::Node::DynamicCreateClosure(func_ref, captures) => {
                todo!("Support CreateClosure in the interpreter")
            }
            ir::Node::StaticCall(_, _) => {
                todo!("Support 'StaticCall' in the interpreter?")
            }
            ir::Node::If(_, _, _) => {
                todo!("Support 'if' in the interpreter")
            }
        };

        (ir::IR { node, location }, typ)
    }
}