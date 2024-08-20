use std::ffi::{c_uint, CString};
use llvm_sys::core::*;
use llvm_sys::LLVMLinkage;
use llvm_sys::prelude::*;
use crate::compiler::function_builder::FunctionBuilder;
use crate::lir;
use crate::ir::Type;

pub struct Compiler<'a> {
    llvm_context: LLVMContextRef,
    llvm_module: LLVMModuleRef,

    lir_module: &'a lir::Module,
    function_declarations: Vec<FunctionDeclaration>
}

pub struct FunctionDeclaration {
    // pub capture_types: Vec<LLVMTypeRef>,
    pub param_types: Vec<LLVMTypeRef>,

    pub type_ref: LLVMTypeRef,
    pub func_ref: LLVMValueRef
}

impl <'a> Compiler<'a> {
    pub fn compile(
        llvm_context: LLVMContextRef,
        llvm_module: LLVMModuleRef,
        lir_module: &'a lir::Module
    ) {
        unsafe {
            let mut compiler = Self {
                llvm_context,
                llvm_module,
                lir_module,
                function_declarations: Vec::with_capacity(lir_module.functions.len())
            };

            compiler.compile_module();
        }
    }

    unsafe fn compile_module(&mut self) {
        // TODO: Make sure we're not trying to compile functions only used during compile-time
        for (i, func) in self.lir_module.functions.iter().enumerate() {
            let name = format!("func_{}", i);
            let decl = self.declare_function(func, &name, false);

            self.function_declarations.push(decl);
        }

        let lir_main = &self.lir_module.main;
        let main_decl = self.declare_function(lir_main, "main", true);

        FunctionBuilder::build(self.llvm_context, self.llvm_module, &main_decl, lir_main);

        // TODO: Make sure we're not trying to compile functions only used during compile-time
        for (i, func) in self.lir_module.functions.iter().enumerate() {
            let decl = &self.function_declarations[i];

            FunctionBuilder::build(self.llvm_context, self.llvm_module, decl, func);
        }
    }

    unsafe fn declare_function(&mut self, func: &lir::Function, name: &str, exported: bool) -> FunctionDeclaration {
        let mut param_types = Vec::with_capacity(func.param_types.len());
        for lir_param in func.param_types.iter() {
            param_types.push(self.llvm_type_of(*lir_param));
        }

        if func.capture_types.len() > 0 {
            todo!("Support closures");
        }

        let llvm_return_type = self.llvm_type_of(func.return_type);

        let type_ref = LLVMFunctionType(
            llvm_return_type,
            param_types.as_mut_ptr(),
            param_types.len() as c_uint,
            0
        );

        let fn_name = CString::new(name).unwrap();
        let func_ref = LLVMAddFunction(self.llvm_module, fn_name.as_ptr(), type_ref);

        if !exported {
            LLVMSetLinkage(func_ref, LLVMLinkage::LLVMInternalLinkage);
        }

        FunctionDeclaration {
            param_types,

            type_ref,
            func_ref
        }
    }

    pub unsafe fn llvm_type_of(&mut self, typ: Type) -> LLVMTypeRef {
        match typ {
            Type::Any => panic!("Cannot represent Any type in runtime-compiled code"),

            // TODO: Represent this using `void`
            Type::None => LLVMInt8TypeInContext(self.llvm_context),
            Type::Bool => LLVMInt8TypeInContext(self.llvm_context),
            Type::Int => LLVMInt64TypeInContext(self.llvm_context),
            Type::Float => LLVMDoubleTypeInContext(self.llvm_context),
            Type::Type => panic!("Cannot represent Type type in runtime-compiled code"),

            Type::Closure(_) => todo!("Support closures"),

            // TODO: We can't use self.function_declarations here since it may not yet be initialized,
            //       since we're using llvm_type_of during initialization
            // Type::Closure(func_ref) => self.function_declarations[func_ref.i].closure_struct_type
            //    // TODO: Support passing non-closure functions by value, maybe another value of type FunctionPtr?
            //    .expect("Function referred to by a closure type did not have a closure struct type")
        }
    }
}