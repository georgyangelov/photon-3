use crate::mir::lexical_scope::*;

/// A simple block scope - it defines new variables in its closest parent stack frame and ensures
/// the defined name is only accessible by the children of the block scope.
pub struct BlockScope {
    /// The referenced names defined in the stack frame but only accessible by children
    names: Vec<(String, BlockNameRef)>
}

#[derive(Copy, Clone)]
pub enum BlockNameRef {
    Local(StackFrameLocalRef),
    Comptime((StackFrameLocalRef, Option<ComptimeExportRef>))
}

impl BlockScope {
    pub fn new() -> Self {
        Self {
            names: Vec::new()
        }
    }

    pub fn set_name(&mut self, name: String, name_ref: BlockNameRef) {
        for (local_name, existing_ref) in self.names.iter_mut() {
            if local_name.as_str() == name {
                *existing_ref = name_ref;
                return
            }
        }

        self.names.push((name, name_ref));
    }

    pub fn find_name(&mut self, name: &str) -> Option<BlockNameRef> {
        let mut local = None;
        for (local_name, stack_ref) in self.names.iter() {
            if local_name == name {
                local = Some(*stack_ref);
                break;
            }
        }

        local
    }
}
