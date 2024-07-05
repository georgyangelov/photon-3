use std::ffi::{c_uint, CString};
use llvm_sys::core::*;
use llvm_sys::LLVMLinkage;
use llvm_sys::prelude::*;
use crate::lir;
use crate::lir::Function;
use crate::types::Type;

pub struct CompilerModuleContext {
    pub context: LLVMContextRef,
    pub module: LLVMModuleRef,

    pub function_declarations: Vec<FunctionDeclaration>
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub capture_types: Vec<LLVMTypeRef>,
    pub param_types: Vec<LLVMTypeRef>,
    pub local_types: Vec<LLVMTypeRef>,

    pub closure_struct_type: Option<LLVMTypeRef>,
    pub type_ref: LLVMTypeRef,
    pub func_ref: LLVMValueRef
}

impl CompilerModuleContext {
    pub fn new(context: LLVMContextRef, module: LLVMModuleRef) -> Self {
        Self {
            context,
            module,

            function_declarations: Vec::new()
        }
    }

    pub unsafe fn declare_functions(&mut self, module: &lir::Module) {
        let mut declarations = Vec::with_capacity(module.functions.len() + 1);

        for (i, func) in module.functions.iter().enumerate() {
            match func {
                None => {
                    // Function is only used during compile-time, we don't want to compile it for runtime
                }

                Some(func) => {
                    let name = format!("func_{i}");
                    let declaration = self.function_declaration(module, func, &name, false);

                    declarations.push(declaration);
                }
            }
        }

        self.function_declarations = declarations;
    }

    pub unsafe fn declare_main(&mut self, module: &lir::Module) -> FunctionDeclaration {
        self.function_declaration(module, &module.main, "main", true)
    }

    unsafe fn function_declaration(
        &mut self,
        module: &lir::Module,
        func: &lir::Function,

        // TODO: Use the name from the lir::Function
        name: &str,

        // TODO: Define this in lir::Function
        exported: bool
    ) -> FunctionDeclaration {
        let closure_struct_type = if func.capture_types.len() > 0 {
            Some(self.llvm_closure_struct_type(module, func))
        } else {
            None
        };

        let mut param_types = Vec::with_capacity(func.param_types.len() + 1);
        for param_type in &func.param_types {
            param_types.push(self.llvm_type_of(&module, *param_type));
        }

        match closure_struct_type {
            None => {}
            Some(closure_struct_type) => param_types.push(closure_struct_type)
        }

        let llvm_return_type = self.llvm_type_of(module, func.return_type);

        let type_ref = LLVMFunctionType(
            llvm_return_type,
            param_types.as_mut_ptr(),
            param_types.len() as c_uint,
            0
        );

        let fn_name = CString::new(name).unwrap();
        let func_ref = LLVMAddFunction(self.module, fn_name.as_ptr(), type_ref);

        if !exported {
            LLVMSetLinkage(func_ref, LLVMLinkage::LLVMInternalLinkage);
        }

        let mut capture_types = Vec::with_capacity(func.capture_types.len());
        for capture_type in &func.capture_types {
            capture_types.push(self.llvm_type_of(module, *capture_type));
        }

        let mut local_types = Vec::with_capacity(func.local_types.len());
        for local_type in &func.local_types {
            local_types.push(self.llvm_type_of(module, *local_type));
        }

        FunctionDeclaration {
            capture_types,
            param_types,
            local_types,

            closure_struct_type,
            type_ref,
            func_ref
        }
    }

    pub unsafe fn llvm_type_of(&mut self, module: &lir::Module, typ: Type) -> LLVMTypeRef {
        match typ {
            Type::Any => panic!("Cannot represent Any type in runtime-compiled code"),
            Type::None => LLVMInt8TypeInContext(self.context),
            Type::Bool => LLVMInt8TypeInContext(self.context),
            Type::Int => LLVMInt64TypeInContext(self.context),
            Type::Float => LLVMDoubleTypeInContext(self.context),
            Type::Type => panic!("Cannot represent Type type in runtime-compiled code"),

            Type::Closure(func_ref) => self.llvm_closure_struct_type(module, &module.functions[func_ref.i].as_ref().unwrap())

            // TODO: We can't use self.function_declarations here since it may not yet be initialized,
            //       since we're using llvm_type_of during initialization
            // Type::Closure(func_ref) => self.function_declarations[func_ref.i].closure_struct_type
            //    // TODO: Support passing non-closure functions by value, maybe another value of type FunctionPtr?
            //    .expect("Function referred to by a closure type did not have a closure struct type")
        }
    }

    unsafe fn llvm_closure_struct_type(
        &mut self,
        module: &lir::Module,
        func: &lir::Function
    ) -> LLVMTypeRef {
        if func.capture_types.len() == 0 {
            return self.llvm_type_of(module, Type::None)
        }

        // We don't want to add the function pointer by default because then the function
        // can be inlined as we won't need its pointer
        let mut struct_fields = Vec::with_capacity(func.capture_types.len());

        for capture_type in &func.capture_types {
            let llvm_type = self.llvm_type_of(module, *capture_type);

            struct_fields.push(llvm_type);
        }

        LLVMStructTypeInContext(
            self.context,
            struct_fields.as_mut_ptr(),
            struct_fields.len() as c_uint,
            0
        )
    }
}