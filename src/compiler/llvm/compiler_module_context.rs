use std::collections::HashMap;
use llvm_sys::core::{LLVMDoubleTypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext};
use llvm_sys::prelude::{LLVMContextRef, LLVMModuleRef, LLVMTypeRef};
use crate::types::Type;

pub struct CompilerModuleContext {
    pub context: LLVMContextRef,
    pub module: LLVMModuleRef,

    type_map: HashMap<Type, LLVMTypeRef>
}

impl CompilerModuleContext {
    pub fn new(context: LLVMContextRef, module: LLVMModuleRef) -> Self {
        Self {
            context,
            module,

            type_map: HashMap::new()
        }
    }

    pub unsafe fn llvm_type_of(&mut self, typ: Type) -> LLVMTypeRef {
        match typ {
            Type::Any => panic!("Cannot represent Any type in runtime-compiled code"),
            Type::None => LLVMInt8TypeInContext(self.context),
            Type::Bool => LLVMInt8TypeInContext(self.context),
            Type::Int => LLVMInt64TypeInContext(self.context),
            Type::Float => LLVMDoubleTypeInContext(self.context),
            Type::Type => panic!("Cannot represent Type type in runtime-compiled code"),
            Type::Closure(_) => todo!("Support closure structs in LLVM")
        }
    }
}