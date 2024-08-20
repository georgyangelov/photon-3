use std::rc::Rc;
use crate::old_lir::{Function, Globals, Instruction, Value, ValueRef};
use crate::{old_lir, mir};
use crate::old_lir::compile_time_state::{CompileTimeState, ResolvedFn};
use crate::types::{IntrinsicFn, Type};

pub struct CompileTimeInterpreter<'a> {
    state: CompileTimeState,

    lir_compiler: old_lir::Compiler<'a>,

    globals: &'a Globals,
    mir_module: &'a mir::Module

    // TODO: Optimize so that all values are part of the same Vec -> data locality
    // stack: Vec<StackFrame>
}

struct StackFrame {
    // globals: Vec<Value>,
    captures: Vec<Value>,
    args: Vec<Value>,
    locals: Vec<Value>
}

impl <'a> CompileTimeInterpreter<'a> {
    pub fn new(globals: &'a Globals, mir_module: &'a mir::Module) -> Self {
        let lir_compiler = old_lir::Compiler::new(globals, mir_module, true);

        let state = CompileTimeState::new(mir_module.comptime_export_count);

        Self {
            state,
            lir_compiler,
            globals,
            mir_module
        }
    }

    pub fn eval(mut self) -> CompileTimeState {
        let main_mir = &self.mir_module.comptime_main;
        let main_lir = self.lir_compiler.compile_main(main_mir, &mut self.state);

        self.eval_func(&main_lir, Vec::new(), Vec::new());

        self.state
    }

    fn eval_func(&mut self, func: &Function, args: Vec<Value>, captures: Vec<Value>) -> Value {
        let mut locals = Vec::new();
        locals.resize(func.local_types.len(), Value::None);

        let mut frame = StackFrame { captures, args, locals };

        self.eval_basic_block(&mut frame, &func.entry)
    }

    fn eval_basic_block(
        &mut self,
        frame: &mut StackFrame,
        block: &old_lir::BasicBlock
    ) -> Value {
        for instruction in &block.code {
            // TODO: Insert type assertions on LIR compilation if the value transitions
            //       from Any to a concrete type
            match instruction {
                Instruction::LocalSet(local_ref, value_ref, _) => {
                    frame.locals[local_ref.i] = self.resolve_value(frame, *value_ref);
                }

                Instruction::CompileTimeSet(export_ref, value_ref, _) => {
                    self.state.comptime_exports[export_ref.i] = self.resolve_value(frame, *value_ref);
                }

                Instruction::CreateDynamicClosure(result_ref, mir_func_ref, capture_types, capture_refs) => {
                    let mir_func = &self.mir_module.functions[mir_func_ref.i];

                    // TODO: Cache this compilation based on the exported types
                    let lir_func_ref = self.lir_compiler.compile_function(mir_func, capture_types.clone(), &mut self.state);

                    // TODO: Do we want to specify the closure types based on the values?
                    //       Do we want those to be more specific than what is known at LIR compile time?
                    let captures = self.resolve_values(frame, capture_refs);

                    let result = Value::Closure(lir_func_ref, Rc::new(captures));

                    frame.locals[result_ref.i] = result;
                }

                Instruction::CreateClosure(result_ref, func_ref, capture_refs) => {
                    let captures = self.resolve_values(frame, capture_refs);

                    let result = Value::Closure(*func_ref, Rc::new(captures));

                    frame.locals[result_ref.i] = result;
                }

                Instruction::CallDynamicFunction(result_ref, name, arg_refs) => {
                    let args = self.resolve_values(frame, arg_refs);
                    let arg_types = self.types_of_values(&args);

                    let result = match &args[0] {
                        // TODO: Do we want this to be an intrinsic?
                        Value::Closure(func_ref, captures) if name == "call" => {
                            let func = self.state.get_compiled_fn(*func_ref);
                            let captures = captures.as_ref().clone();

                            // TODO: Is it better to not clone the args here?
                            let args_without_self = args[1..].to_vec();

                            self.eval_func(func.as_ref(), args_without_self, captures)
                        }

                        _ => {
                            let resolved_fn = self.state.resolve_fn(name, &arg_types);

                            match resolved_fn {
                                None => panic!("Could not find function {} on arg types {:?}", name, &arg_types),
                                Some(ResolvedFn::Intrinsic(intrinsic)) => self.call_intrinsic(intrinsic, args),
                                Some(ResolvedFn::Function(_)) => {
                                    // TODO: Get the mir function reference, compile the function and call it
                                    todo!("Support dynamic function calls")
                                }
                            }
                        }
                    };

                    frame.locals[result_ref.i] = result;
                }

                Instruction::CallIntrinsicFunction(result_ref, intrinsic_fn, arg_refs, _) => {
                    let args = self.resolve_values(frame, arg_refs);

                    let result = self.call_intrinsic(*intrinsic_fn, args);

                    frame.locals[result_ref.i] = result;
                }

                Instruction::CallStaticFunction(result_ref, func_ref, arg_refs, _) => {
                    let args = self.resolve_values(frame, arg_refs);
                    let func = self.state.get_compiled_fn(*func_ref);

                    let result = self.eval_func(func.as_ref(), args, Vec::new());

                    frame.locals[result_ref.i] = result;
                }

                Instruction::CallPtrFunction(_, _, _, _) => todo!("Support calling function pointers"),
                Instruction::CallPtrClosureFunction(_, _, _, _, _) => todo!("Support calling closures through function pointers"),

                Instruction::CallStaticClosureFunction(result_ref, func_ref, closure_ref, arg_refs, _) => {
                    let closure_struct = self.resolve_value(frame, *closure_ref);
                    let args = self.resolve_values(frame, arg_refs);

                    let (closure_func_ref, captures) = closure_struct.assert_closure();
                    let captures = captures.clone();
                    assert_eq!(closure_func_ref, *func_ref);

                    let func = self.state.get_compiled_fn(*func_ref);

                    let result = self.eval_func(func.as_ref(), args, captures);

                    frame.locals[result_ref.i] = result;
                }

                Instruction::Return(value_ref, _) => {
                    return self.resolve_value(frame, *value_ref);
                }

                Instruction::If(_, _, _, _) => {}
            };
        }

        Value::None
    }

    fn call_intrinsic(&mut self, intrinsic: IntrinsicFn, args: Vec<Value>) -> Value {
        match intrinsic {
            IntrinsicFn::AddInt => Value::Int(args[0].assert_int() + args[1].assert_int())
        }
    }

    fn types_of_values(&self, values: &[Value]) -> Vec<Type> {
        let mut types = Vec::with_capacity(values.len());

        for value in values {
            let typ = value.type_of();

            types.push(typ);
        }

        types
    }

    fn resolve_values(&self, frame: &StackFrame, value_refs: &[ValueRef]) -> Vec<Value> {
        let mut values = Vec::with_capacity(value_refs.len());

        for arg_ref in value_refs {
            let value = self.resolve_value(frame, *arg_ref);

            values.push(value);
        }

        values
    }

    fn resolve_value(&self, frame: &StackFrame, value_ref: ValueRef) -> Value {
        match value_ref {
            ValueRef::None => Value::None,
            ValueRef::Bool(value) => Value::Bool(value),
            ValueRef::Int(value) => Value::Int(value),
            ValueRef::Float(value) => Value::Float(value),
            ValueRef::Global(global_ref) => self.globals.globals[global_ref.i].value.clone(),
            ValueRef::ComptimeExport(export_ref) => self.state.comptime_exports[export_ref.i].clone(),
            ValueRef::Const(_) => todo!("Constants in interpreter"),
            ValueRef::Capture(capture_ref) => frame.captures[capture_ref.i].clone(),
            ValueRef::Param(param_ref) => frame.args[param_ref.i].clone(),
            ValueRef::Local(local_ref) => frame.locals[local_ref.i].clone(),
        }
    }
}