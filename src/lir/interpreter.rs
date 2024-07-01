use crate::lir::{Function, Instruction, Value, ValueRef};
use crate::{lir, mir};
use crate::types::IntrinsicFn;

pub struct CompileTimeInterpreter<'a> {
    comptime_exports: Vec<Value>,
    lir_compiler: lir::Compiler,

    mir_module: &'a mir::Module

    // TODO: Optimize so that all values are part of the same Vec -> data locality
    // stack: Vec<StackFrame>
}

struct StackFrame<'a> {
    func: &'a Function,
    // globals: Vec<Value>,
    args: Vec<Value>,
    locals: Vec<Value>
}

impl <'a> CompileTimeInterpreter<'a> {
    pub fn new(mir_module: &'a mir::Module) -> Self {
        let mut comptime_exports = Vec::new();

        // TODO: Make sure exports are not used before being defined
        comptime_exports.resize(mir_module.comptime_export_count, Value::None);

        let lir_compiler = lir::Compiler::new(false);

        Self {
            comptime_exports,
            lir_compiler,
            mir_module
        }
    }

    pub fn eval(mut self) -> Vec<Value> {
        let main_mir = &self.mir_module.comptime_main;
        let main_lir = self.lir_compiler.compile_main(main_mir, &self.comptime_exports);

        self.eval_func(&main_lir, Vec::new());

        self.comptime_exports
    }

    fn eval_func(&mut self, func: &Function, args: Vec<Value>) -> Value {
        let mut locals = Vec::new();
        locals.resize(func.local_types.len(), Value::None);
        let mut frame = StackFrame { func, args, locals };

        self.eval_basic_block(&mut frame, &func.entry)
    }

    fn eval_basic_block(
        &mut self,
        frame: &mut StackFrame,
        block: &lir::BasicBlock
    ) -> Value {
        for instruction in &block.code {
            // TODO: Insert type assertions on LIR compilation if the value transitions
            //       from Any to a concrete type
            match instruction {
                Instruction::LocalSet(local_ref, value_ref, _) => {
                    frame.locals[local_ref.i] = self.resolve_value(frame, *value_ref);
                }

                Instruction::CompileTimeSet(export_ref, value_ref, _) => {
                    self.comptime_exports[export_ref.i] = self.resolve_value(frame, *value_ref);
                }

                Instruction::CallIntrinsicFunction(result_local_ref, intrinsic_fn, arg_refs, _) => {
                    let mut args = Vec::with_capacity(arg_refs.len());
                    for arg_ref in arg_refs {
                        let value = self.resolve_value(frame, *arg_ref);

                        args.push(value);
                    }

                    let result = match intrinsic_fn {
                        IntrinsicFn::AddInt => Value::Int(args[0].assert_int() + args[1].assert_int())
                    };

                    frame.locals[result_local_ref.i] = result;
                }

                Instruction::Return(value_ref, _) => {
                    return self.resolve_value(frame, *value_ref);
                }

                Instruction::If(_, _, _, _) => {}
            };
        }

        Value::None
    }

    fn resolve_value(&self, frame: &StackFrame, value_ref: ValueRef) -> Value {
        match value_ref {
            ValueRef::None => Value::None,
            ValueRef::Bool(value) => Value::Bool(value),
            ValueRef::Int(value) => Value::Int(value),
            ValueRef::Float(value) => Value::Float(value),
            ValueRef::ComptimeExport(export_ref) => self.comptime_exports[export_ref.i].clone(),
            ValueRef::Const(_) => todo!("Constants in interpreter"),
            ValueRef::Param(param_ref) => frame.args[param_ref.i].clone(),
            ValueRef::Local(local_ref) => frame.locals[local_ref.i].clone(),
        }
    }
}