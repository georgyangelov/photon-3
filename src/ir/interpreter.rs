use std::rc::Rc;
use crate::{ir, lir};
use crate::ir::{CaptureFrom, Globals, StaticCapture, Type, Value};

pub struct Interpreter<'a> {
    globals: &'a Globals,

    // This is Rc because while specializing (referencing) a function from here, we also need
    // to hold a mutable reference to the Interpreter struct.
    ir_functions: Vec<Rc<ir::Function>>,

    lir_functions: Vec<lir::Function>
}

#[derive(Debug)]
struct ComptimeStackFrame {
    params: Vec<ParamInfo>,
    captures: Vec<CaptureInfo>,
    locals: Vec<LocalInfo>,
    runtime_local_count: usize,

    return_type: Option<Type>
}

#[derive(Debug)]
struct StackFrameType {
    typ: Type,
    comptime: bool
}

#[derive(Debug, Clone)]
struct ParamInfo {
    typ: Type,
    comptime_value: Option<Value>,
    runtime_ref: Option<lir::ParamRef>,
}

#[derive(Debug, Clone)]
struct CaptureInfo {
    typ: Type,
    comptime_value: Option<Value>,
    runtime_ref: Option<lir::CaptureRef>
}

#[derive(Debug, Clone)]
struct LocalInfo {
    typ: Option<Type>,
    comptime_value: Option<Value>,
    runtime_ref: Option<lir::LocalRef>
}

impl <'a> Interpreter<'a> {
    pub fn eval_comptime(globals: &'a Globals, module: ir::Module) -> lir::Module {
        let mut interpreter = Self {
            globals,
            ir_functions: module.functions.into_iter().map(|func| Rc::new(func)).collect(),
            lir_functions: Vec::new()
        };
        let main = interpreter.specialize_function(&module.main, Vec::new(), Vec::new());

        lir::Module { main, functions: interpreter.lir_functions }
    }

    fn specialize_function(
        &mut self,
        func: &ir::Function,
        mut params: Vec<ParamInfo>,
        mut captures: Vec<CaptureInfo>
    ) -> lir::Function {
        // TODO: If the function body resolves to a constant - return that constant directly
        //       instead of wrapping in a function?

        let mut lir_i = 0;
        for (i, param) in func.params.iter().enumerate() {
            if param.comptime {
                if params[i].comptime_value.is_none() {
                    panic!("Must pass all comptime values at compile-time")
                }
            } else {
                params[i].runtime_ref = Some(lir::ParamRef { i: lir_i });
                lir_i += 1;
            }
        }

        let mut lir_i = 0;
        for (i, capture) in func.captures.iter().enumerate() {
            if capture.comptime {
                if captures[i].comptime_value.is_none() {
                    panic!("Must pass all comptime captures at compile-time")
                }
            } else {
                captures[i].runtime_ref = Some(lir::CaptureRef { i: lir_i });
                lir_i += 1;
            }
        }

        let mut locals = Vec::with_capacity(func.locals.len());
        let mut runtime_local_count = 0;
        for local in &func.locals {
            // TODO: This can be dynamic
            let runtime_ref = if local.comptime {
                None
            } else {
                let lir_ref = lir::LocalRef { i: runtime_local_count };
                runtime_local_count += 1;
                Some(lir_ref)
            };

            locals.push(LocalInfo {
                runtime_ref,
                typ: None,
                comptime_value: None
            });
        }

        let mut frame = ComptimeStackFrame {
            params,
            captures,
            locals,
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
        let (return_ref, body_typ) = self.specialize_ir(&mut frame, &mut body, &func.body, false);

        let return_type = match return_type {
            None => body_typ,
            Some(typ) => {
                todo!("Compare return_type with body_type")
            }
        };

        // TODO: Better void type
        body.code.push(lir::Instruction::Return(return_ref));

        // NOTE: This assumes that runtime params/captures are ordered the same way as the full list
        let runtime_param_types = frame.params.into_iter()
            .filter_map(|param| match param.runtime_ref {
                None => None,
                Some(_) => Some(param.typ)
            })
            .collect();

        // NOTE: This assumes that runtime params/captures are ordered the same way as the full list
        let runtime_capture_types = frame.captures.into_iter()
            .filter_map(|param| match param.runtime_ref {
                None => None,
                Some(_) => Some(param.typ)
            })
            .collect();

        lir::Function {
            capture_types: runtime_capture_types,
            param_types: runtime_param_types,
            return_type,
            local_count: frame.runtime_local_count,
            body
        }
    }

    fn specialize_ir(
        &mut self,
        frame: &mut ComptimeStackFrame,
        block: &mut lir::BasicBlock,
        ir: &ir::IR,
        comptime: bool
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
                    if comptime {
                        todo!("panic")
                    }

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
                    if comptime {
                        todo!("panic")
                    }

                    let param_info = &frame.params[param_ref.i];
                    let lir_param_ref = param_info.runtime_ref.unwrap();
                    let value_ref = lir::ValueRef::Param(lir_param_ref);
                    let typ = param_info.typ.clone();

                    (value_ref, typ)
                }
            }
            ir::Node::LocalRef(local_ref) => {
                let local_info = &frame.locals[local_ref.i];
                let typ = local_info.typ.clone().unwrap();

                let value_ref = match local_info.runtime_ref {
                    None => {
                        // Comptime local
                        let value = local_info.comptime_value.as_ref().expect("Missing comptime local value");

                        Self::value_to_lir(value)
                    }
                    Some(local_ref) => {
                        // Runtime local
                        if comptime {
                            todo!("panic")
                        }

                        lir::ValueRef::Local(local_ref)
                    }
                };

                (value_ref, typ)
            }
            ir::Node::CaptureRef(_) => todo!("Support specializing captures"),
            ir::Node::LocalSet(local_ref, value_ir) => {
                let (value_ref, typ) = self.specialize_ir(frame, block, value_ir, local_ref.comptime);

                let local_info = &mut frame.locals[local_ref.i];
                local_info.typ = Some(typ.clone());

                if local_ref.comptime {
                    let value = Self::assert_const(value_ref);

                    if typ.is_any() {
                        todo!("Support specializing Any types");
                    }

                    local_info.comptime_value = Some(value);
                } else {
                    if comptime {
                        todo!("panic")
                    }

                    let lir_local_ref = local_info.runtime_ref.expect("Missing runtime ref for local");
                    let instruction = lir::Instruction::LocalSet(lir_local_ref, value_ref, typ);

                    block.code.push(instruction);
                }

                (lir::ValueRef::None, Type::None)
            }
            ir::Node::Block(irs) => {
                let mut result = (lir::ValueRef::None, Type::None);
                for ir in irs {
                    result = self.specialize_ir(frame, block, ir, comptime);
                }

                result
            }
            ir::Node::Comptime(_) => todo!("Support specializing and calling comptime blocks"),
            ir::Node::Call(name, target, args) => {
                if comptime {
                    todo!("Call functions")
                }

                let (target_ref, target_type) = self.specialize_ir(frame, block, target, comptime);
                let (mut arg_refs, mut arg_types) = self.specialize_args_with_target(
                    frame,
                    block,
                    // TODO: Eliminate this clone somehow?
                    (target_ref, target_type.clone()),
                    args,
                    comptime
                );

                let resolved_fn = match (target_type, name.as_ref()) {
                    (Type::Any, _) => panic!("Target type cannot be Any"),
                    (Type::None, _) => todo!("Support calling functions on None"),
                    (Type::Bool, _) => todo!("Support calling functions on bools"),
                    (Type::Int, "+") => ResolvedFn::Intrinsic(ir::IntrinsicFn::AddInt),
                    (Type::Float, _) => todo!("Support calling functions on floats"),
                    (Type::Type, _) => todo!("Support calling functions on types"),
                    (Type::StaticClosure(func_ref, captures), "call") => {
                        let func = self.ir_functions[func_ref.i].clone();

                        // TODO: Make this better
                        arg_refs.remove(0);
                        arg_types.remove(0);

                        let mut param_info = Vec::with_capacity(func.params.len());
                        for (i, param) in func.params.iter().enumerate() {
                            let comptime_value = if param.comptime {
                                Some(Self::assert_const(arg_refs[i]))
                            } else {
                                None
                            };

                            param_info.push(ParamInfo {
                                // TODO: Remove this clone
                                typ: arg_types[i].clone(),
                                comptime_value,

                                // Will be populated by `specialize_function`
                                runtime_ref: None,
                            })
                        }

                        let mut capture_info = Vec::with_capacity(func.captures.len());
                        for (i, capture) in captures.into_iter().enumerate() {
                            let comptime_value = if func.captures[i].comptime {
                                Some(capture.comptime_value.clone().unwrap())
                            } else {
                                None
                            };

                            capture_info.push(CaptureInfo {
                                typ: capture.typ,
                                comptime_value,

                                // Will be populated by `specialize_function`
                                runtime_ref: None
                            })
                        }

                        // TODO: Same signatures should specialize to the same lir::FunctionRef
                        let specialized_fn = self.specialize_function(&func, param_info, capture_info);

                        self.lir_functions.push(specialized_fn);
                        let lir_func_ref = lir::FunctionRef { i: self.lir_functions.len() - 1 };

                        ResolvedFn::Function(lir_func_ref)
                    }

                    (typ, name) => panic!("Cannot find function {} on {:?}", name, typ)
                };

                let signature = match &resolved_fn {
                    ResolvedFn::Intrinsic(intrinsic) => intrinsic.signature(&arg_types),
                    ResolvedFn::Function(func_ref) => {
                        let func = &self.lir_functions[func_ref.i];

                        // TODO: Move this to LIR and make it part of Function
                        ir::FunctionSignature {
                            params: func.param_types.clone(),
                            returns: func.return_type.clone()
                        }
                    }
                };

                let result_local_ref = Self::new_temp_local(frame);

                let instruction = match resolved_fn {
                    ResolvedFn::Intrinsic(intrinsic) => lir::Instruction::CallIntrinsic(result_local_ref, intrinsic, arg_refs),
                    ResolvedFn::Function(func_ref) => lir::Instruction::CallStaticClosure(result_local_ref, func_ref, target_ref, arg_refs)
                };

                block.code.push(instruction);

                (lir::ValueRef::Local(result_local_ref), signature.returns)
            }
            ir::Node::CreateClosure(func_ref) => {
                let func = &self.ir_functions[func_ref.i];

                let mut static_captures = Vec::with_capacity(func.captures.len());
                let mut runtime_capture_value_refs = Vec::with_capacity(func.captures.len());

                if func.params.iter().any(|p| p.comptime) && !comptime {
                    // Do we want to actually support this?
                    todo!("Cannot create closure with comptime params at runtime")
                }

                // TODO: Tidy up this code
                for capture in &func.captures {
                    match capture.from {
                        CaptureFrom::Capture(capture_ref) => {
                            if capture.comptime {
                                todo!("Support capture of comptime captures")
                                // let value = match frame.capture_values.get(&capture_ref) {
                                //     None => panic!("Could not find compile-time capture"),
                                //     Some(value) => value
                                // }.clone();
                                // comptime_capture_values.push(value);
                            } else {
                                todo!("Support capture of capture refs")
                            }

                            // let typ = frame.capture_types[capture_ref.i].clone();
                            // capture_types.push(typ);
                        }

                        CaptureFrom::Param(param_ref) => todo!("Support capture of param refs"),

                        CaptureFrom::Local(local_ref) => {
                            let local_info = &frame.locals[local_ref.i];

                            let comptime_value = if capture.comptime {
                                let value = match &local_info.comptime_value {
                                    None => panic!("Could not find compile-time local"),
                                    Some(value) => value
                                }.clone();

                                Some(value)
                            } else {
                                let value_ref = lir::ValueRef::Local(local_info.runtime_ref.unwrap());

                                runtime_capture_value_refs.push(value_ref);

                                None
                            };

                            static_captures.push(StaticCapture {
                                typ: local_info.typ.clone().unwrap(),
                                comptime_value
                            })
                        }
                    };
                    // let value = self.specialize_ir(frame, block, capture)
                }

                let closure_local_ref = Self::new_temp_local(frame);

                let closure_type = Type::StaticClosure(*func_ref, static_captures);
                let closure_instr = lir::Instruction::CreateStaticClosure(closure_local_ref, runtime_capture_value_refs);
                block.code.push(closure_instr);

                (lir::ValueRef::Local(closure_local_ref), closure_type)
            },
            ir::Node::If(_, _, _) => todo!("Support specializing ifs")
        }
    }

    #[inline]
    fn specialize_args_with_target(
        &mut self,
        frame: &mut ComptimeStackFrame,
        block: &mut lir::BasicBlock,
        target: (lir::ValueRef, Type),
        args: &[ir::IR],
        comptime: bool
    ) -> (Vec<lir::ValueRef>, Vec<Type>) {
        let mut refs = Vec::with_capacity(args.len() + 1);
        let mut types = Vec::with_capacity(args.len() + 1);

        refs.push(target.0);
        types.push(target.1);

        for arg in args {
            let (value_ref, value_type) = self.specialize_ir(frame, block, arg, comptime);

            refs.push(value_ref);
            types.push(value_type);
        }

        (refs, types)
    }

    #[inline]
    fn specialize_args_without_target(
        &mut self,
        frame: &mut ComptimeStackFrame,
        block: &mut lir::BasicBlock,
        args: &[ir::IR],
        comptime: bool
    ) -> (Vec<lir::ValueRef>, Vec<Type>) {
        let mut refs = Vec::with_capacity(args.len());
        let mut types = Vec::with_capacity(args.len());

        for arg in args {
            let (value_ref, value_type) = self.specialize_ir(frame, block, arg, comptime);

            refs.push(value_ref);
            types.push(value_type);
        }

        (refs, types)
    }

    fn new_temp_local(frame: &mut ComptimeStackFrame) -> lir::LocalRef {
        let i = frame.runtime_local_count;
        frame.runtime_local_count += 1;

        lir::LocalRef { i }
    }

    fn assert_const(value_ref: lir::ValueRef) -> Value {
        match value_ref {
            lir::ValueRef::None => Value::None,
            lir::ValueRef::Bool(value) => Value::Bool(value),
            lir::ValueRef::Int(value) => Value::Int(value),
            lir::ValueRef::Float(value) => Value::Float(value),
            lir::ValueRef::Param(_) => todo!("Error handling"),
            lir::ValueRef::Local(_) => todo!("Error handling")
        }
    }

    fn value_to_lir(value: &Value) -> lir::ValueRef {
        match value {
            Value::None => lir::ValueRef::None,
            Value::Bool(value) => lir::ValueRef::Bool(*value),
            Value::Int(value) => lir::ValueRef::Int(*value),
            Value::Float(value) => lir::ValueRef::Float(*value),
            Value::Type(_) => todo!("Support referencing types from runtime code?"),
            Value::StaticClosure(_) => todo!("Support closure exports")
        }
    }
}

#[derive(Clone)]
pub enum ResolvedFn {
    Intrinsic(ir::IntrinsicFn),
    Function(lir::FunctionRef)
}