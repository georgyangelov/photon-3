use crate::compiler::lexical_scope::*;

/// A simple block scope - it defines new variables in its closest parent stack frame and ensures
/// the defined name is only accessible by the children of the block scope.
pub struct BlockScope<'a> {
    parent: &'a mut dyn LexicalScope,

    /// The referenced names defined in the stack frame but only accessible by children
    names: Vec<(String, BlockNameRef)>
}

#[derive(Copy, Clone)]
enum BlockNameRef {
    Local(StackFrameLocalRef),
    Comptime((StackFrameLocalRef, Option<ComptimeExportRef>))
}

impl <'a> BlockScope<'a> {
    pub fn new(parent: &'a mut dyn LexicalScope) -> Self {
        Self {
            parent,
            names: Vec::new()
        }
    }

    pub fn new_child_block(&mut self) -> BlockScope {
        BlockScope::new(self)
    }

    pub fn new_child_stack_frame(&mut self) -> StackFrame {
        StackFrame::new(self)
    }

    pub fn new_child_comptime_portal(&'a mut self) -> ComptimePortal<'a> {
        ComptimePortal::new(self)
    }

    pub fn define_local(&mut self, name: String) -> StackFrameLocalRef {
        let stack_ref = self.parent.define_stack_frame_local();

        self.names.push((name, BlockNameRef::Local(stack_ref)));

        stack_ref
    }

    pub fn define_comptime_main_local(&mut self, name: String) -> StackFrameLocalRef {
        let comptime_main_stack_ref = self.parent.define_comptime_main_stack_frame_local();

        self.names.push((name, BlockNameRef::Comptime((comptime_main_stack_ref, None))));

        comptime_main_stack_ref
    }

    pub fn access_local(&mut self, name: &str) -> Result<NameRef, NameAccessError> {
        // By default, code is runtime, so we need to access comptime vals through exports.
        // However, If we pass through a ComptimePortal, then this will get changed to `false`.
        self.access_name(name, true)
    }
}

impl <'a> LexicalScope for BlockScope<'a> {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.parent.define_comptime_main_stack_frame_local()
    }

    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.parent.define_stack_frame_local()
    }

    fn define_comptime_export(&mut self) -> ComptimeExportRef {
        self.parent.define_comptime_export()
    }

    fn access_name(&mut self, name: &str, export_comptime: bool) -> Result<NameRef, NameAccessError> {
        let mut local = None;
        for (i, (local_name, stack_ref)) in self.names.iter().enumerate() {
            if local_name == name {
                local = Some((i, *stack_ref));
                break;
            }
        }

        match local {
            None => self.parent.access_name(name, export_comptime),
            Some((_, BlockNameRef::Local(local_ref))) => Ok(NameRef::Local(local_ref)),
            Some((i, BlockNameRef::Comptime((local_ref, export_ref)))) => {
                if export_comptime {
                    if let Some(export_ref) = export_ref {
                        Ok(NameRef::ComptimeExport(export_ref))
                    } else {
                        let export_ref = self.parent.define_comptime_export();

                        let new_value = BlockNameRef::Comptime((local_ref, Some(export_ref)));
                        let _ = std::mem::replace(&mut self.names[i], (String::from(name), new_value));

                        Ok(NameRef::ComptimeExport(export_ref))
                    }
                } else {
                    Ok(NameRef::ComptimeLocal(local_ref))
                }
            },
        }
    }
}