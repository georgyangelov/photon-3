use crate::compiler::lexical_scope::*;

/// This indicates a switch to compile-time code. It acts as a portal to the ComptimeMainStackFrame
/// above it. If a new variable is defined by any block inside, it will not affect the current
/// StackFrame but ComptimeMainStackFrame.
///
/// For example:
///
///     // Root -> ComptimeMainStackFrame [ a ] -> StackFrame [ fn, c ] -> BlockScope
///     // vals: comptime [ a ], local [ fn, c ]
///
///     val c = ...
///
///     @val a = (
///       // Root -> ComptimeMainStackFrame -> StackFrame -> BlockScope -> ComptimePortal -> BlockScope
///       // vals: comptime [ ], local [ b ]
///
///       // This variable is defined in the comptime stack frame instead of the main runtime one
///       val b = 42
///       b
///
///       // If we try to access c here, it should be an error - how?
///     )
///
///     val fn = @{ // this is a ComptimePortal -> StackFrame -> BlockScope
///       // Root -> ComptimeMainStackFrame -> StackFrame -> BlockScope -> ComptimePortal -> StackFrame [ b ] -> BlockScope
///       // vals: comptime [ ], local [ b ]
///
///       // Can access `a`, but needs to capture it
///       a
///
///       // This variable is defined in fn's stack frame, which inherits from the comptime one
///       val b = 42
///
///       (
///         // This is a child BlockScope of fn, any vars defined here still get defined in
///         // fn's stack frame
///         42
///
///         // Can access b directly
///       )
///
///       @(
///         // This is ComptimePortal -> BlockScope again. Any variables defined here will be
///         // defined in the comptime main stack frame, instead of fn's stack frame.
///         42
///
///         // Cannot access b
///       )
///
///       {
///         b + 41
///
///         // Can access b, needs to capture it
///       }
///     }
///
pub struct ComptimePortal<'a> {
    parent: &'a mut BlockScope<'a>
}

impl <'a> ComptimePortal<'a> {
    pub fn new(parent: &'a mut BlockScope<'a>) -> Self {
        ComptimePortal { parent }
    }

    pub fn new_child_block(&mut self) -> BlockScope {
        BlockScope::new(self)
    }
}

impl <'a> LexicalScope for ComptimePortal<'a> {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.parent.define_comptime_main_stack_frame_local()
    }

    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.parent.define_comptime_main_stack_frame_local()
    }

    fn define_comptime_export(&mut self) -> ComptimeExportRef {
        self.parent.define_comptime_export()
    }

    fn access_name(&mut self, name: &str, _export_comptime: bool) -> Result<NameRef, NameAccessError> {
        let parent_ref = self.parent.access_name(name, false)?;

        match parent_ref {
            NameRef::Global(global_ref) => Ok(NameRef::Global(global_ref)),
            NameRef::ComptimeExport(_) => todo!("This shouldn't happen"),
            NameRef::ComptimeLocal(local_ref) => Ok(NameRef::Local(local_ref)),
            NameRef::Local(_) => Err(NameAccessError::CannotReferenceRuntimeNameFromComptime)
        }
    }
}
