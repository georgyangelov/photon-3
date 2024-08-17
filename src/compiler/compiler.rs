use crate::ir;

pub struct JITCompiler {

}

impl JITCompiler {
    pub fn new(module: &ir::PostComptimeModule) -> Self {
        Self {}
    }

    pub fn compile<T>(&mut self) -> (unsafe extern fn () -> T) {
        todo!("Support compilation of IR")
    }
}