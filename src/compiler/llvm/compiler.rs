use crate::lir;

pub struct JITCompiler<'a> {
    module: &'a lir::Module
}

impl <'a> JITCompiler<'a> {
    pub fn new(module: &'a lir::Module) -> Self {
        Self { module }
    }

    pub fn compile(&mut self) -> unsafe extern "C" fn() -> lir::Value {
        todo!("Implement compiler")
    }
}
