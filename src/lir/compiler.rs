use std::collections::HashMap;
use crate::mir;
use crate::lir::*;
use crate::lir::Instruction::Return;
use crate::types::{IntrinsicLookup, Type};

pub struct Compiler {
    intrinsic_lookup: IntrinsicLookup,

    comptime_exports: Vec<Value>,
    constants: Vec<Value>,

    // TODO: Maybe use arenas for these functions?
    functions: Vec<CompilingFunction>,

    // struct_types: Arena<Type>,
    // interface_types: Arena<Type>

    func_map: HashMap<mir::FunctionRef, FunctionRef>,
    export_map: HashMap<mir::ComptimeExportRef, ConstRef>
}

enum CompilingFunction {
    Pending,
    Compiled(Function)
}

struct FunctionBuilder {
    param_types: Vec<Type>,
    local_types: Vec<Type>,
    return_type: Type
}

impl Compiler {
    pub fn compile(mir: &mir::Module, comptime_exports: Vec<Value>) -> Module {
        let mut compiler = Compiler {
            intrinsic_lookup: IntrinsicLookup::new(),

            comptime_exports,
            constants: vec![],

            functions: Vec::new(),

            func_map: HashMap::new(),
            export_map: HashMap::new(),
        };

        let main = compiler.compile_function(&mir.runtime_main);

        let mut functions = Vec::with_capacity(compiler.functions.len());
        for func in compiler.functions {
            functions.push(match func {
                CompilingFunction::Pending => panic!("Non-compiled function still present in vec"),
                CompilingFunction::Compiled(func) => func
            });
        }

        Module {
            constants: compiler.constants,
            functions,
            main
        }
    }

    fn compile_function(&mut self, func: &mir::Function) -> Function {
        let mut param_types = Vec::with_capacity(func.param_types.len());
        for param_type in &func.param_types {
            param_types.push(self.read_exported_type(*param_type));
        }

        let return_type = self.read_exported_type(func.return_type);

        let mut local_types = Vec::with_capacity(func.local_count);
        local_types.resize(func.local_count, Type::None);

        let mut builder = FunctionBuilder {
            param_types,
            return_type,
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
            return_type: builder.return_type,
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
                let const_ref = if self.export_map.contains_key(export_ref) {
                    self.export_map[export_ref]
                } else {
                    let const_ref = ConstRef { i: self.constants.len() };

                    // TODO: Verify value is serializable
                    self.constants.push(self.comptime_exports[export_ref.i].clone());
                    self.export_map.insert(*export_ref, const_ref);

                    const_ref
                };

                let value = &self.constants[const_ref.i];

                (ValueRef::Const(const_ref), value.type_of())
            }

            mir::Node::CompileTimeSet(_, _) => panic!("Cannot have CompileTimeSet instructions in runtime code"),

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
                    let intrinsic = self.intrinsic_lookup.find(target_type, name);
                    match intrinsic {
                        None => panic!("Cannot find function {} on type {:?}", name, target_type),
                        Some((intrinsic_fn, fn_type)) => {
                            // TODO: Type-check the arguments
                            // TODO: Insert conversion operators here, then call the function

                            let result_type = fn_type.returns;
                            let result_ref = new_temp_local(builder, fn_type.returns);

                            let instruction = Instruction::CallIntrinsicFunction(result_ref, *intrinsic_fn, arg_refs, fn_type.returns);
                            block.code.push(instruction);

                            (ValueRef::Local(result_ref), result_type)
                        }
                    }
                } else {
                    todo!("Support dynamic function calls on Any")
                }
            }

            mir::Node::CreateClosure(_, _) => todo!("Support closures"),
            mir::Node::If(_, _, _) => todo!("Support ifs")
        }
    }

    fn compile_and_add_function(&mut self, mir_ref: mir::FunctionRef, func: &mir::Function) -> FunctionRef {
        let func_ref = FunctionRef { i: self.functions.len() };

        self.func_map.insert(mir_ref, func_ref);
        self.functions.push(CompilingFunction::Pending);

        let func = self.compile_function(func);

        self.functions[func_ref.i] = CompilingFunction::Compiled(func);
        func_ref
    }

    fn read_exported_type(&self, export: Option<mir::ComptimeExportRef>) -> Type {
        match export {
            // TODO: Verify that this Any is not present for runtime functions
            None => Type::Any,
            Some(export_ref) => {
                let value = &self.comptime_exports[export_ref.i];

                match value {
                    Value::Type(typ) => *typ,
                    // TODO: Location
                    _ => panic!("Invalid value specified as a type, got {:?}", value)
                }
            }
        }
    }
}

fn new_temp_local(builder: &mut FunctionBuilder, typ: Type) -> LocalRef {
    let i = builder.local_types.len();

    builder.local_types.push(typ);

    LocalRef { i }
}