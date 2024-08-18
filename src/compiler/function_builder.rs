use llvm_sys::prelude::*;
use crate::compiler::compiler::FunctionDeclaration;
use crate::ir;

pub struct FunctionBuilder {

}

impl FunctionBuilder {
    pub unsafe fn build(
        llvm_context: LLVMContextRef,
        llvm_module: LLVMModuleRef,

        decl: &FunctionDeclaration,
        func: &ir::RFunction
    ) -> FunctionDeclaration {
        todo!("Ability to build functions")
    }
}