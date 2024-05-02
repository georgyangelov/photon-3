use crate::compiler::lexical_scope::*;

/// The compile-time main function. There is only one such scope per-module and it executes the
/// compile-time code.
///
/// It's also the only one that can export data from the compile-time code to the run-time code.
/// It does that by allocating "comptime export slots" on the RootScope, then copying locals from
/// the stack frame into the export slot (usually global memory), which is later included in the
/// runtime binary as static data and can be referenced from there.
pub struct ComptimeMainStackFrame<'a> {
    parent: &'a mut RootScope,

    /// The local stack frame slots - these would only be used during comptime run
    locals: Vec<StackFrameLocal>,

    /// Tracks local slots which need to be exported
    exports: Vec<(StackFrameLocalRef, ComptimeExportRef)>
}

impl <'a> ComptimeMainStackFrame<'a> {
    pub fn new(parent: &'a mut RootScope) -> Self {
        Self {
            parent,
            locals: Vec::new(),
            exports: Vec::new()
        }
    }

    pub fn new_block(&mut self) -> BlockScope {
        BlockScope::new(self)
    }
}

impl <'a> LexicalScope for ComptimeMainStackFrame<'a> {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.define_stack_frame_local()
    }

    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        let i = self.locals.len();

        self.locals.push(StackFrameLocal {});

        StackFrameLocalRef { i }
    }

    fn define_comptime_export(&mut self) -> ComptimeExportRef {
        self.parent.define_comptime_export()
    }

    fn access_name(&mut self, name: &str, export_comptime: bool) -> Result<NameRef, NameAccessError> {
        self.parent.access_name(name, export_comptime)
    }
}