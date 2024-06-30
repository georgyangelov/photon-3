use std::collections::HashMap;
use crate::mir;
use crate::lir::*;
use crate::types::Type;

pub struct Compiler {
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
        let mut entry = BasicBlock {
            code: Vec::new()
        };

        // let param_types = func.

        let mut builder = FunctionBuilder {
            param_types: todo!(),
            local_types: todo!(),
            return_type: todo!()
        };

        self.compile_mir(&mut builder, &mut entry, func, &func.body);

        Function { entry }
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
            mir::Node::ParamRef(param_ref) => (ValueRef::Param(ParamRef { i: param_ref.i }), todo!("Get param type")),

            mir::Node::CaptureRef(_) => todo!("Support capture struct"),

            mir::Node::LocalGet(local_ref) => (ValueRef::Local(LocalRef { i: local_ref.i }), todo!("Infer local types")),
            mir::Node::LocalSet(local_ref, mir) => {
                let local_ref = LocalRef { i: local_ref.i };
                let (value_ref, typ) = self.compile_mir(builder, block, func, mir);

                block.code.push(Instruction::LocalSet(local_ref, value_ref, typ));

                (ValueRef::None, Type::None)
            }

            mir::Node::Block(_) => todo!("Support blocks"),
            mir::Node::Call(_, _, _) => todo!("Support function calls"),
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
}
