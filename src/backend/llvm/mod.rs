use crate::compiler;

struct LLVMCompiler<'a> {
    mir_module: &'a compiler::Module,
}

impl <'a> LLVMCompiler<'a> {
    pub fn new(mir_module: &'a compiler::Module) -> Self {
        LLVMCompiler { mir_module }
    }

    pub fn compile_to_wasm(&mut self) -> Vec<u8> {
        todo!()
    }
}