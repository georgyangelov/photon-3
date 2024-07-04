use std::collections::HashMap;
use std::rc::Rc;
use crate::mir;
use crate::lir::*;
use crate::lir::compile_time_state::{CompileTimeState, CompilingFunction, ResolvedFn};
use crate::lir::Instruction::Return;
use crate::mir::lexical_scope::CaptureFrom;
use crate::types::{Type, IntrinsicFn, FunctionSignature};
use crate::types::IntrinsicFn::{AddInt, CallClosure};

pub struct Compiler<'a> {
    mir_module: &'a mir::Module,
    exports_are_constants: bool,

    constants: Vec<Value>,

    // TODO: Maybe use arenas for these functions?
    // functions: Vec<CompilingFunction>,

    // func_map: HashMap<mir::FunctionRef, FunctionRef>,
    export_const_map: HashMap<mir::ComptimeExportRef, ConstRef>
}

struct FunctionBuilder<'a> {
    state: &'a mut CompileTimeState,
    param_types: Vec<Type>,
    local_types: Vec<Type>
}

impl <'a> Compiler<'a> {
    pub fn new(
        mir_module: &'a mir::Module,
        exports_are_constants: bool
    ) -> Self {
        Self {
            mir_module,
            exports_are_constants,
            constants: Vec::new(),
            // func_map: HashMap::new(),
            export_const_map: HashMap::new()
        }
    }

    pub fn compile(
        mir_module: &'a mir::Module,
        mut state: CompileTimeState
    ) -> Module {
        let mut compiler = Self::new(mir_module, true);

        let main = compiler.compile_function_mir(&mir_module.runtime_main, &mut state);

        let mut functions = Vec::with_capacity(state.functions.len());
        for func in state.functions {
            functions.push(match func {
                CompilingFunction::Pending => panic!("Non-compiled function still present in vec"),
                CompilingFunction::Compiled(func) => Rc::try_unwrap(func).unwrap()
            });
        }

        Module {
            constants: compiler.constants,
            functions,
            main
        }
    }

    // TODO: Remove this function?
    pub fn compile_main(
        &mut self,
        func: &mir::Function,
        state: &mut CompileTimeState
    ) -> Function {
        self.compile_function_mir(func, state)
    }

    pub fn compile_function(
        &mut self,
        func: &mir::Function,
        state: &mut CompileTimeState
    ) -> FunctionRef {
        let func_ref = FunctionRef { i: state.functions.len() };

        state.functions.push(CompilingFunction::Pending);

        let func = self.compile_function_mir(func, state);

        state.functions[func_ref.i] = CompilingFunction::Compiled(Rc::new(func));
        func_ref
    }

    fn compile_function_mir(
        &mut self,
        func: &mir::Function,
        state: &mut CompileTimeState
    ) -> Function {
        let mut param_types = Vec::with_capacity(func.param_types.len());
        for param_type in &func.param_types {
            let param_type = self.read_exported_type(*param_type, state);

            param_types.push(param_type.unwrap_or(Type::Any));
        }

        let return_type = self.read_exported_type(func.return_type, state);

        let mut local_types = Vec::with_capacity(func.local_count);
        local_types.resize(func.local_count, Type::None);

        let mut builder = FunctionBuilder {
            state,
            param_types,
            local_types,
        };

        let mut entry = BasicBlock {
            code: Vec::new()
        };

        let (value_ref, typ) = self.compile_mir(&mut builder, &mut entry, func, &func.body);

        // TODO: Type-check the type with builder.return_type
        entry.code.push(Return(value_ref, typ));

        Function {
            param_types: builder.param_types,
            return_type: return_type.unwrap_or(typ),
            local_types: builder.local_types,
            entry
        }
    }

    fn compile_mir(
        &mut self,
        builder: &mut FunctionBuilder,
        block: &mut BasicBlock,
        func: &mir::Function,
        mir: &mir::MIR
    ) -> (ValueRef, Type) {
        match &mir.node {
            mir::Node::Nop => (ValueRef::None, Type::None),
            mir::Node::CompileTimeGet(export_ref) => {
                if self.exports_are_constants {
                    let export_ref = if self.export_const_map.contains_key(export_ref) {
                        self.export_const_map[export_ref]
                    } else {
                        let const_ref = ConstRef { i: self.constants.len() };

                        // TODO: Verify value is serializable
                        self.constants.push(builder.state.comptime_exports[export_ref.i].clone());
                        self.export_const_map.insert(*export_ref, const_ref);

                        const_ref
                    };

                    let value = &self.constants[export_ref.i];

                    (ValueRef::Const(export_ref), value.type_of())
                } else {
                    // TODO: Can the type of this be inferred on the CompileTimeSet node?
                    (ValueRef::ComptimeExport(*export_ref), Type::Any)
                }
            }

            mir::Node::CompileTimeSet(export_ref, mir) => {
                if self.exports_are_constants {
                    panic!("Cannot compile CompileTimeSet if exports are constants")
                }

                let (value_ref, value_type) = self.compile_mir(builder, block, func, mir);

                let instr = Instruction::CompileTimeSet(*export_ref, value_ref, value_type);
                block.code.push(instr);

                (ValueRef::None, Type::None)
            },

            mir::Node::GlobalRef(_) => todo!("Support GlobalRef"),
            mir::Node::ConstStringRef(_) => todo!("Support ConstStringRef"),
            mir::Node::LiteralBool(value) => (ValueRef::Bool(*value), Type::Bool),
            mir::Node::LiteralI64(value) => (ValueRef::Int(*value), Type::Int),
            mir::Node::LiteralF64(value) => (ValueRef::Float(*value), Type::Float),
            mir::Node::ParamRef(param_ref) => (ValueRef::Param(ParamRef { i: param_ref.i }), builder.param_types[param_ref.i]),

            mir::Node::CaptureRef(_) => todo!("Support capture struct"),

            mir::Node::LocalGet(local_ref) => (ValueRef::Local(LocalRef { i: local_ref.i }), builder.local_types[local_ref.i]),
            mir::Node::LocalSet(local_ref, mir) => {
                let local_ref = LocalRef { i: local_ref.i };
                let (value_ref, typ) = self.compile_mir(builder, block, func, mir);

                builder.local_types[local_ref.i] = typ;

                block.code.push(Instruction::LocalSet(local_ref, value_ref, typ));

                (ValueRef::None, Type::None)
            }

            mir::Node::Block(mirs) => {
                let mut result = (ValueRef::None, Type::None);

                for mir in mirs {
                    result = self.compile_mir(builder, block, func, mir);
                }

                result
            }

            mir::Node::Call(name, target, args) => {
                let (target_ref, target_type) = self.compile_mir(builder, block, func, target);

                let mut arg_refs = Vec::with_capacity(args.len() + 1);
                let mut arg_types = Vec::with_capacity(args.len() + 1);

                arg_refs.push(target_ref);
                arg_types.push(target_type);

                for arg in args {
                    let (arg_ref, arg_type) = self.compile_mir(builder, block, func, arg);

                    arg_refs.push(arg_ref);
                    arg_types.push(arg_type);
                }

                if target_type != Type::Any {
                    // Concrete type, function can be determined statically
                    // TODO: Template functions can't be determined statically
                    // TODO: Support non-intrinsic functions
                    let resolved_func = builder.state.resolve_fn(name, &arg_types);
                    match resolved_func {
                        None => panic!("Cannot find function {} on type {:?}", name, target_type),
                        Some(ResolvedFn::Intrinsic(intrinsic)) => {
                            // TODO: Type-check the arguments
                            // TODO: Insert conversion operators here, then call the function

                            let fn_type = self.intrinsic_signature(builder.state, intrinsic, &arg_types);
                            let result_type = fn_type.returns;
                            let result_ref = new_temp_local(builder, result_type);

                            let instr = Instruction::CallIntrinsicFunction(result_ref, intrinsic, arg_refs, result_type);
                            block.code.push(instr);

                            (ValueRef::Local(result_ref), result_type)
                        }
                        Some(ResolvedFn::Function(_)) => todo!("Support non-intrinsic static functions")
                    }
                } else {
                    let result_type = Type::Any;
                    let result_ref = new_temp_local(builder, result_type);

                    let instr = Instruction::CallDynamicFunction(result_ref, name.to_string(), arg_refs, result_type);
                    block.code.push(instr);

                    (ValueRef::Local(result_ref), result_type)
                }
            }

            mir::Node::CreateClosure(func_ref, captures) => {
                let mir_func = &self.mir_module.functions[func_ref.i];
                let lir_func_ref = self.compile_function(mir_func, &mut builder.state);

                let closure_type = Type::Closure(lir_func_ref);
                let temp_local_ref = new_temp_local(builder, closure_type);

                let mut capture_refs = Vec::with_capacity(captures.len());
                for capture in captures {
                    capture_refs.push(self.resolve_capture_ref(*capture));
                }

                block.code.push(Instruction::CreateClosure(temp_local_ref, lir_func_ref, capture_refs));

                (ValueRef::Local(temp_local_ref), closure_type)
            }

            mir::Node::If(_, _, _) => todo!("Support ifs")
        }
    }

    fn resolve_capture_ref(&self, capture: CaptureFrom) -> ValueRef {
        match capture {
            CaptureFrom::Capture(mir_capture_ref) => ValueRef::Capture(CaptureRef { i: mir_capture_ref.i }),
            CaptureFrom::Param(mir_param_ref) => ValueRef::Param(ParamRef { i: mir_param_ref.i }),
            CaptureFrom::Local(mir_local_ref) => ValueRef::Local(LocalRef { i: mir_local_ref.i })
        }
    }

    fn read_exported_type(&self, export: Option<mir::ComptimeExportRef>, state: &CompileTimeState) -> Option<Type> {
        match export {
            // TODO: Verify that this Any is not present for runtime functions
            None => None,
            Some(export_ref) => {
                let value = &state.comptime_exports[export_ref.i];

                match value {
                    Value::Type(typ) => Some(*typ),
                    // TODO: Location
                    _ => panic!("Invalid value specified as a type, got {:?}", value)
                }
            }
        }
    }

    // PERFORMANCE: Optimize to not create new objects every time
    // TODO: Does this need to be in the TypeRegistry?
    fn intrinsic_signature(&self, state: &CompileTimeState, intrinsic: IntrinsicFn, arg_types: &[Type]) -> FunctionSignature {
        match intrinsic {
            AddInt => FunctionSignature { params: vec![Type::Int, Type::Int], returns: Type::Int },
            CallClosure => {
                let lir_func_ref = match arg_types[0] {
                    Type::Closure(lir_func_ref) => lir_func_ref,
                    _ => panic!("Cannot call CallClosure on something that's not a closure")
                };
                // let mir_func = &self.mir_module.functions[func_ref.i];
                //
                // let lir_func_ref = self.compile_function(*func_ref, mir_func, comptime_exports, type_registry);
                let lir_func = &state.get_compiled_fn(lir_func_ref);

                FunctionSignature { params: lir_func.param_types.clone(), returns: lir_func.return_type }
            }
        }
    }
}

fn new_temp_local(builder: &mut FunctionBuilder, typ: Type) -> LocalRef {
    let i = builder.local_types.len();

    builder.local_types.push(typ);

    LocalRef { i }
}