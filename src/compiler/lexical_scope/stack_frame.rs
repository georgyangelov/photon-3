use crate::compiler::lexical_scope::*;

/// A function/closure scope - it has locals and can reference variables from parent scopes by
/// capturing them.
pub struct StackFrame<'a> {
    parent: &'a mut dyn LexicalScope,

    /// The captured values from parent scopes
    pub captures: Vec<Capture>,

    /// The local stack frame slots
    locals: Vec<StackFrameLocal>,
}

impl <'a> StackFrame<'a> {
    pub fn new(parent: &'a mut dyn LexicalScope) -> Self {
        Self {
            parent,
            captures: Vec::new(),
            locals: Vec::new()
        }
    }

    pub fn new_child_block(&mut self) -> BlockScope {
        BlockScope::new(self)
    }
}

impl <'a> LexicalScope for StackFrame<'a> {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.parent.define_comptime_main_stack_frame_local()
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
        match self.parent.access_name(name, export_comptime)? {
            NameRef::Local(parent_local_ref) => {
                let child_local_ref = self.define_stack_frame_local();

                self.captures.push(Capture {
                    from: parent_local_ref,
                    to: child_local_ref
                });

                Ok(NameRef::Local(child_local_ref))
            },

            comptime_ref @ NameRef::ComptimeExport(_) => Ok(comptime_ref),
            local_ref @ NameRef::ComptimeLocal(_) => Ok(local_ref),
            global_ref @ NameRef::Global(_) => Ok(global_ref),
        }
    }
}