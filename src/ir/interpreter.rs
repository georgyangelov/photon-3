use crate::{ir, lir};
use crate::ir::{Globals, Type, Value};
use crate::vec_map::VecMap;

pub struct Interpreter<'a> {
    globals: &'a Globals,
    functions: Vec<lir::Function>
}

struct ComptimeStackFrame {
    // TODO: Optimization for the lookups of these VecMaps here?
    param_types: Vec<Type>,
    param_values: VecMap<ir::ParamRef, Value>,
    runtime_param_map: VecMap<ir::ParamRef, lir::ParamRef>,

    capture_types: Vec<Type>,
    capture_values: VecMap<ir::CaptureRef, Value>,

    local_types: VecMap<ir::LocalRef, StackFrameType>,
    local_values: VecMap<ir::LocalRef, Value>,
    runtime_local_map: VecMap<ir::LocalRef, lir::LocalRef>,
    runtime_local_count: usize,

    return_type: Option<Type>
}

struct StackFrameType {
    typ: Type,
    comptime: bool
}

impl <'a> Interpreter<'a> {
    pub fn eval_comptime(globals: &'a Globals, module: ir::Module) -> lir::Module {
        let mut interpreter = Self {
            globals,
            functions: Vec::new()
        };
        let main = interpreter.specialize_function(
            module.main,
            Vec::new(),
            VecMap::new(),
            Vec::new(),
            VecMap::new()
        );

        lir::Module { main, functions: interpreter.functions }
    }

    fn specialize_function(
        &mut self,

        // TODO: Pass a closure here instead of raw FunctionTemplate, we need the types for the
        //       specialization
        func: ir::Function,

        param_types: Vec<Type>,
        comptime_param_values: VecMap<ir::ParamRef, Value>,

        capture_types: Vec<Type>,
        comptime_capture_values: VecMap<ir::CaptureRef, Value>
    ) -> lir::Function {
        // TODO: If the function body resolves to a constant - return that constant directly
        //       instead of wrapping in a function?

        let mut runtime_param_map = VecMap::with_capacity(func.params.len());
        let mut lir_i = 0;
        for (ir_i, param) in func.params.iter().enumerate() {
            if !param.comptime {
                runtime_param_map.insert_push(
                    ir::ParamRef { i: ir_i, comptime: param.comptime },
                    lir::ParamRef { i: lir_i }
                );
                lir_i += 1;
            }
        }

        let mut runtime_local_map = VecMap::with_capacity(func.locals.len());
        let mut lir_i = 0;
        for (ir_i, local) in func.locals.iter().enumerate() {
            if !local.comptime {
                runtime_local_map.insert_push(
                    ir::LocalRef { i: ir_i, comptime: local.comptime },
                    lir::LocalRef { i: lir_i }
                );
                lir_i += 1;
            }
        }

        let runtime_local_count = runtime_local_map.len();

        let mut stack_frame = ComptimeStackFrame {
            param_types,
            param_values: comptime_param_values,
            runtime_param_map,

            capture_types,
            capture_values: comptime_capture_values,

            local_types: VecMap::new(),
            local_values: VecMap::new(),
            runtime_local_map,
            runtime_local_count,

            return_type: None
        };

        // TODO: Eval & type-check arguments

        let return_type = match &func.return_type {
            None => None,
            Some(ir) => {
                todo!("Support comptime eval");

                Some(Type::None)
                // let (value, _) = self.eval_ir(&mut stack_frame, ir, true);
                // let value = self.assert_const_value(value);
                //
                // Some(match value {
                //     Value::Type(typ) => typ,
                //     value => panic!("The return value expression must evaluate to a type, got {:?}", value)
                // })
            }
        };

        let mut body = lir::BasicBlock { code: Vec::new() };
        let (return_ref, body_typ) = self.specialize_ir(&mut stack_frame, &mut body, &func.body);

        let return_type = match return_type {
            None => body_typ,
            Some(typ) => {
                todo!("Compare return_type with body_type")
            }
        };

        // TODO: Better void type
        body.code.push(lir::Instruction::Return(return_ref));

        let mut runtime_param_types = Vec::new();
        for (i, param) in func.params.iter().enumerate() {
            if !param.comptime {
                runtime_param_types.push(stack_frame.param_types[i]);
            }
        }

        let mut runtime_capture_types = Vec::new();
        for (i, capture) in func.captures.iter().enumerate() {
            if !capture.comptime {
                runtime_capture_types.push(stack_frame.capture_types[i]);
            }
        }

        lir::Function {
            capture_types: runtime_capture_types,
            param_types: runtime_param_types,
            return_type,
            local_count: stack_frame.runtime_local_count,
            body
        }
    }

    fn assert_const_value(&mut self, ir: ir::IR) -> Value {
        match ir.node {
            ir::Node::Nop => Value::None,
            ir::Node::Constant(value) => value,
            result => panic!("Could not fully eval IR, got {:?}", result)
        }
    }

    fn specialize_ir(
        &mut self,
        frame: &mut ComptimeStackFrame,
        block: &mut lir::BasicBlock,
        ir: &ir::IR
    ) -> (lir::ValueRef, Type) {
        let location = ir.location.clone();

        match &ir.node {
            ir::Node::Nop => (lir::ValueRef::None, Type::None),
            ir::Node::Constant(value) => {
                let value_ref = Self::value_to_lir(value);

                (value_ref, value.type_of())
            }
            ir::Node::GlobalRef(global_ref) => {
                let value_ref = if global_ref.comptime {
                    todo!("Resolve global value as constant");
                    todo!("Support specializing Any types");
                } else {
                    // ir::Node::GlobalRef(*global_ref)
                    todo!("Support global refs")
                };

                todo!()
            }
            ir::Node::ParamRef(param_ref) => {
                if param_ref.comptime {
                    todo!("Resolve param value as constant");
                    todo!("Support specializing Any types");
                } else {
                    let lir_param_ref = *frame.runtime_param_map.get(param_ref).expect("Missing param map");
                    let value_ref = lir::ValueRef::Param(lir_param_ref);
                    let typ = frame.param_types[param_ref.i];

                    (value_ref, typ)
                }
            }
            ir::Node::LocalRef(local_ref) => {
                let frame_type = frame.local_types.get(local_ref)
                    .expect("Used local before assignment");

                if frame_type.comptime {
                    if frame_type.typ == Type::Any {
                        todo!("Support specializing Any types");
                    }

                    let value = frame.local_values.get(local_ref)
                        .expect("Used local before assignment");

                    (Self::value_to_lir(value), value.type_of())
                } else {
                    let lir_local_ref = *frame.runtime_local_map.get(local_ref).expect("Missing local map");
                    let value_ref = lir::ValueRef::Local(lir_local_ref);
                    let typ = frame.local_types.get(local_ref).expect("Used param before definition").typ;

                    (value_ref, typ)
                }
            }
            ir::Node::CaptureRef(_) => todo!("Support specializing captures"),
            ir::Node::LocalSet(local_ref, value_ir) => {
                let (value_ref, typ) = self.specialize_ir(frame, block, value_ir);

                if local_ref.comptime {
                    todo!("Run the specialized IR and evaluate to a Value")

                    // if typ == Type::Any {
                    //     todo!("Support specializing Any types");
                    // }
                    //
                    // let value = self.assert_const_value(value);
                    //
                    // frame.local_values.insert(*local_ref, value);
                    //
                    // ir::Node::Nop
                } else {
                    let lir_local_ref = *frame.runtime_local_map.get(local_ref).expect("Missing local map");
                    let instruction = lir::Instruction::LocalSet(lir_local_ref, value_ref, typ);

                    block.code.push(instruction);
                }

                frame.local_types.insert(*local_ref, StackFrameType {
                    typ,
                    comptime: local_ref.comptime
                });

                (lir::ValueRef::None, Type::None)
            }
            ir::Node::Block(irs) => {
                let mut result = (lir::ValueRef::None, Type::None);
                for ir in irs {
                    result = self.specialize_ir(frame, block, ir);
                }

                result
            }
            ir::Node::Comptime(_) => todo!("Support specializing and calling comptime blocks"),
            ir::Node::Call(name, target, args) => {
                let (target_ref, target_type) = self.specialize_ir(frame, block, target);

                let mut arg_types = Vec::with_capacity(args.len() + 1);
                let mut arg_refs = Vec::with_capacity(args.len() + 1);

                // TODO: Remove this clone
                arg_types.push(target_type.clone());
                arg_refs.push(target_ref);

                for arg in args {
                    let (value_ref, value_type) = self.specialize_ir(frame, block, arg);

                    arg_refs.push(value_ref);
                    arg_types.push(value_type);
                }

                let resolved_fn = match (target_type, name.as_ref()) {
                    (Type::Any, _) => panic!("Target type cannot be Any"),
                    (Type::None, _) => todo!("Support calling functions on None"),
                    (Type::Bool, _) => todo!("Support calling functions on bools"),
                    (Type::Int, "+") => ResolvedFn::Intrinsic(ir::IntrinsicFn::AddInt),
                    (Type::Float, _) => todo!("Support calling functions on floats"),
                    (Type::Type, _) => todo!("Support calling functions on types"),
                    (Type::Closure(_), _) => todo!("Support calling functions on closures"),
                    (typ, name) => panic!("Cannot find function {} on {:?}", name, typ)
                };

                let signature = match &resolved_fn {
                    ResolvedFn::Intrinsic(intrinsic) => intrinsic.signature(&arg_types),
                    ResolvedFn::TFunction(_) => todo!("Support getting signature of TFunctions"),
                    ResolvedFn::RFunction(_) => todo!("Support getting signature of RFunctions")
                };

                let result_local_ref = Self::new_temp_local(frame);

                let instruction = match resolved_fn {
                    ResolvedFn::Intrinsic(intrinsic) => lir::Instruction::CallIntrinsic(result_local_ref, intrinsic, arg_refs),
                    ResolvedFn::TFunction(_) => todo!("Support calling TFunctions"),
                    ResolvedFn::RFunction(_) => todo!("Support calling RFunctions")
                };

                block.code.push(instruction);

                (lir::ValueRef::Local(result_local_ref), signature.returns)
            }
            ir::Node::CreateClosure(_, _) => todo!("Support specializing closures"),
            ir::Node::If(_, _, _) => todo!("Support specializing ifs")
        }
    }

    fn new_temp_local(frame: &mut ComptimeStackFrame) -> lir::LocalRef {
        let i = frame.runtime_local_count;
        frame.runtime_local_count += 1;

        lir::LocalRef { i }
    }

    // fn eval_ir(
    //     &mut self,
    //     frame: &mut ComptimeStackFrame,
    //     block: &mut lir::BasicBlock,
    //     ir: &ir::IR,
    //     in_comptime: bool
    // ) -> (lir::ValueRef, Type) {
    //     let location = ir.location.clone();
    //     let (node, typ) = match &ir.node {
    //         ir::Node::Nop => (lir::ValueRef::None, Type::None),
    //         ir::Node::Constant(value) => {
    //             let value_ref = match value {
    //                 Value::None => lir::ValueRef::None,
    //                 Value::Bool(value) => lir::ValueRef::Bool(*value),
    //                 Value::Int(value) => lir::ValueRef::Int(*value),
    //                 Value::Float(value) => lir::ValueRef::Float(*value),
    //                 Value::Type(_) => todo!("Support referencing types from runtime code?"),
    //                 Value::Closure(_, _) => todo!("Support closure exports")
    //             };
    //
    //             (value_ref, value.type_of())
    //         }
    //         ir::Node::GlobalRef(global_ref) => {
    //             let value_ref = if global_ref.comptime {
    //                 todo!("Resolve global value as constant");
    //                 todo!("Support specializing Any types");
    //             } else {
    //                 // ir::Node::GlobalRef(*global_ref)
    //                 todo!("Support global refs")
    //             };
    //
    //             // (value_ref, _)
    //         }
    //         ir::Node::ParamRef(param_ref) => {
    //             let ir = if param_ref.comptime {
    //                 todo!("Resolve param value as constant");
    //                 todo!("Support specializing Any types");
    //             } else {
    //                 ir::Node::ParamRef(*param_ref)
    //             };
    //
    //             (ir, *frame.param_types.get(param_ref).expect("Missing param type"))
    //         }
    //         ir::Node::LocalRef(local_ref) => {
    //             let frame_type = frame.local_types.get(local_ref)
    //                 .expect("Used local before assignment");
    //
    //             let ir = if frame_type.comptime {
    //                 if frame_type.typ == Type::Any {
    //                     todo!("Support specializing Any types");
    //                 }
    //
    //                 let value = frame.local_values.get(local_ref)
    //                     .expect("Used local before assignment");
    //
    //                 ir::Node::Constant(value.clone())
    //             } else {
    //                 ir::Node::LocalRef(*local_ref)
    //             };
    //
    //             (ir, frame.local_types.get(local_ref).expect("Used param before definition").typ)
    //         }
    //         ir::Node::CaptureRef(capture_ref) => {
    //             let ir = if capture_ref.comptime {
    //                 todo!("Resolve captured value as constant");
    //                 todo!("Support specializing Any types");
    //             } else {
    //                 ir::Node::CaptureRef(*capture_ref)
    //             };
    //
    //             (ir, *frame.capture_types.get(capture_ref).expect("Missing capture type"))
    //         }
    //         ir::Node::LocalSet(local_ref, value_ir) => {
    //             let (value, typ) = self.eval_ir(frame, value_ir, local_ref.comptime);
    //
    //             let ir = if local_ref.comptime {
    //                 if typ == Type::Any {
    //                     todo!("Support specializing Any types");
    //                 }
    //
    //                 let value = self.assert_const_value(value);
    //
    //                 frame.local_values.insert(*local_ref, value);
    //
    //                 ir::Node::Nop
    //             } else {
    //                 ir::Node::LocalSet(*local_ref, Box::new(value))
    //             };
    //
    //             frame.local_types.insert(*local_ref, StackFrameType {
    //                 typ,
    //                 comptime: local_ref.comptime
    //             });
    //
    //             (ir, typ)
    //         }
    //         ir::Node::Block(irs) => {
    //             if irs.len() == 0 {
    //                 (ir::Node::Constant(Value::None), Type::None)
    //             } else {
    //                 let mut results = Vec::with_capacity(irs.len());
    //                 let mut result_type = Type::None;
    //                 for ir in irs {
    //                     let (ir, typ) = self.eval_ir(frame, ir, in_comptime);
    //                     result_type = typ;
    //
    //                     // TODO: Remove if it resolves to a constant and it's not the last expression
    //                     results.push(ir);
    //                 }
    //
    //                 (ir::Node::Block(results), result_type)
    //             }
    //         }
    //         ir::Node::Comptime(ir) => {
    //             let (result_ir, result_typ) = self.eval_ir(frame, ir, true);
    //             let value = self.assert_const_value(result_ir);
    //
    //             // Doing this because of the location & to make sure we checked that it's fully
    //             // evaluated
    //             (ir::Node::Constant(value), result_typ)
    //         },
    //         ir::Node::Call(name, target, args) => {
    //             let (target, target_type) = self.eval_ir(frame, target, in_comptime);
    //
    //             let mut arg_types = Vec::with_capacity(args.len() + 1);
    //             let mut arg_irs = Vec::with_capacity(args.len() + 1);
    //
    //             // TODO: Remove this clones
    //             arg_types.push(target_type.clone());
    //             arg_irs.push(target);
    //
    //             for arg in args {
    //                 let (ir, typ) = self.eval_ir(frame, arg, in_comptime);
    //
    //                 arg_irs.push(ir);
    //                 arg_types.push(typ);
    //             }
    //
    //             let resolved_fn = match (target_type, name.as_ref()) {
    //                 (Type::Any, _) => panic!("Target type cannot be Any"),
    //                 (Type::None, _) => todo!("Support calling functions on None"),
    //                 (Type::Bool, _) => todo!("Support calling functions on bools"),
    //                 (Type::Int, "+") => ResolvedFn::Intrinsic(ir::IntrinsicFn::AddInt),
    //                 (Type::Float, _) => todo!("Support calling functions on floats"),
    //                 (Type::Type, _) => todo!("Support calling functions on types"),
    //                 (Type::Closure(_), _) => todo!("Support calling functions on closures"),
    //                 (typ, name) => panic!("Cannot find function {} on {:?}", name, typ)
    //             };
    //
    //             let signature = match &resolved_fn {
    //                 ResolvedFn::Intrinsic(intrinsic) => intrinsic.signature(&arg_types),
    //                 ResolvedFn::TFunction(_) => todo!("Support getting signature of TFunctions"),
    //                 ResolvedFn::RFunction(_) => todo!("Support getting signature of RFunctions")
    //             };
    //
    //             let node = match resolved_fn {
    //                 ResolvedFn::Intrinsic(intrinsic) => ir::Node::StaticCallIntrinsic(intrinsic, arg_irs),
    //                 ResolvedFn::TFunction(_) => todo!("Support calling TFunctions"),
    //                 ResolvedFn::RFunction(_) => todo!("Support calling RFunctions")
    //             };
    //
    //             (node, signature.returns)
    //         }
    //         ir::Node::CreateClosure(func_ref, captures) => {
    //             todo!("Support CreateClosure in the interpreter")
    //         }
    //         ir::Node::StaticCallIntrinsic(_, _) => {
    //             todo!("Support 'StaticCallIntrinsic' in the interpreter")
    //         }
    //         ir::Node::StaticCall(_, _) => {
    //             todo!("Support 'StaticCall' in the interpreter?")
    //         }
    //         ir::Node::If(_, _, _) => {
    //             todo!("Support 'if' in the interpreter")
    //         }
    //     };
    //
    //     (ir::IR { node, location }, typ)
    // }

    fn value_to_lir(value: &Value) -> lir::ValueRef {
        match value {
            Value::None => lir::ValueRef::None,
            Value::Bool(value) => lir::ValueRef::Bool(*value),
            Value::Int(value) => lir::ValueRef::Int(*value),
            Value::Float(value) => lir::ValueRef::Float(*value),
            Value::Type(_) => todo!("Support referencing types from runtime code?"),
            Value::Closure(_, _) => todo!("Support closure exports")
        }
    }
}

#[derive(Clone, Copy)]
pub enum ResolvedFn {
    Intrinsic(ir::IntrinsicFn),
    TFunction(ir::FunctionTemplateRef),
    RFunction(ir::FunctionRef)
}