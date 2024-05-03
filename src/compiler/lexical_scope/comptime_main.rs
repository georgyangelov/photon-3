use crate::compiler::lexical_scope::*;

/// The compile-time main function. There is only one such scope per-module and it executes the
/// compile-time code.
///
/// It's also the only one that can export data from the compile-time code to the run-time code.
/// It does that by allocating "comptime export slots" on the RootScope, then copying locals from
/// the stack frame into the export slot (usually global memory), which is later included in the
/// runtime binary as static data and can be referenced from there.
pub struct ComptimeMainStackFrame {
    /// The local stack frame slots - these would only be used during comptime run
    pub locals: Vec<StackFrameLocal>,

    /// Tracks local slots which need to be exported
    pub exports: Vec<(StackFrameLocalRef, ComptimeExportRef)>
}

impl ComptimeMainStackFrame {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            exports: Vec::new()
        }
    }

    pub fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        let i = self.locals.len();

        self.locals.push(StackFrameLocal {});

        StackFrameLocalRef { i }
    }
}