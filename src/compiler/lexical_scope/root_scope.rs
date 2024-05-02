use crate::compiler::lexical_scope::*;

pub struct RootScope {
    // runtime_globals: Vec<Global>,
    // comptime_globals: Vec<Global>,

    comptime_exports: Vec<ComptimeExportSlot>,
}

impl RootScope {
    pub fn new() -> Self {
        Self {
            // runtime_globals: Vec::new(),
            // comptime_globals: Vec::new(),
            comptime_exports: Vec::new()
        }
    }

    pub fn new_comptime_main_frame(&mut self) -> ComptimeMainStackFrame {
        ComptimeMainStackFrame::new(self)
    }
}

impl LexicalScope for RootScope {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        panic!("This should not happen - missing ComptimeMainStackFrame in scope chain")
    }

    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        panic!("This should not happen - missing StackFrame in scope chain")
    }

    fn define_comptime_export(&mut self) -> ComptimeExportRef {
        let i = self.comptime_exports.len();

        self.comptime_exports.push(ComptimeExportSlot {});

        ComptimeExportRef { i }
    }

    fn access_name(&mut self, _name: &str, _export_comptime: bool) -> Result<NameRef, NameAccessError> {
        Err(NameAccessError::NameNotFound)
    }
}