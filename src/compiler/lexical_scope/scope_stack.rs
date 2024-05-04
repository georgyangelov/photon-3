use crate::compiler::lexical_scope::*;

pub struct ScopeStack {
    stack: Vec<Scope>
}

pub enum Scope {
    RootScope(RootScope),
    ComptimeMainFrame(ComptimeMainStackFrame),
    ComptimePortal(ComptimePortal),
    StackFrame(StackFrame),
    BlockScope(BlockScope)
}

impl ScopeStack {
    pub fn new(root: RootScope, comptime_frame: ComptimeMainStackFrame) -> Self {
        ScopeStack {
            stack: vec![
                Scope::RootScope(root),
                Scope::ComptimeMainFrame(comptime_frame)
            ]
        }
    }

    pub fn push(&mut self, scope: Scope) {
        self.stack.push(scope)
    }

    pub fn push_block(&mut self) {
        let scope = BlockScope::new();

        self.stack.push(Scope::BlockScope(scope))
    }

    pub fn push_stack_frame(&mut self) {
        let scope = StackFrame::new();

        self.stack.push(Scope::StackFrame(scope))
    }

    pub fn push_comptime_portal(&mut self) {
        let portal = ComptimePortal::new();

        self.stack.push(Scope::ComptimePortal(portal))
    }

    pub fn pop(&mut self) -> Scope {
        match self.stack.pop() {
            None => panic!("Attempted to pop more scopes than exist"),
            Some(scope) => scope
        }
    }

    pub fn pop_root(&mut self) -> RootScope {
        match self.pop() {
            Scope::RootScope(root) => root,
            _ => panic!("Expected a root scope")
        }
    }

    pub fn pop_comptime_main_stack_frame(&mut self) -> ComptimeMainStackFrame {
        match self.pop() {
            Scope::ComptimeMainFrame(frame) => frame,
            _ => panic!("Expected a comptime main stack frame")
        }
    }

    pub fn pop_comptime_portal(&mut self) -> ComptimePortal {
        match self.pop() {
            Scope::ComptimePortal(portal) => portal,
            _ => panic!("Expected a comptime portal")
        }
    }

    pub fn pop_stack_frame(&mut self) -> StackFrame {
        match self.pop() {
            Scope::StackFrame(frame) => frame,
            _ => panic!("Expected a stack frame")
        }
    }

    pub fn pop_block(&mut self) -> BlockScope {
        match self.pop() {
            Scope::BlockScope(scope) => scope,
            _ => panic!("Expected a block scope")
        }
    }

    pub fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        match &mut self.stack[1] {
            Scope::ComptimeMainFrame(frame) => frame.define_stack_frame_local(),
            _ => panic!("Expected the second element to be the comptime main stack frame")
        }
    }

    pub fn define_comptime_export(&mut self) -> ComptimeExportRef {
        match &mut self.stack[0] {
            Scope::RootScope(root) => root.define_comptime_export(),
            _ => panic!("Expected the first element to be the root scope")
        }
    }

    pub fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        let mut should_define_in_comptime_main = false;

        for i in (0..self.stack.len()).rev() {
            match &mut self.stack[i] {
                Scope::RootScope(_) => {},
                Scope::ComptimeMainFrame(frame) => {
                    if should_define_in_comptime_main {
                        return frame.define_stack_frame_local()
                    }
                },
                Scope::StackFrame(frame) => {
                    if !should_define_in_comptime_main {
                        return frame.define_stack_frame_local()
                    }
                },
                Scope::BlockScope(_) => {}
                Scope::ComptimePortal(_) => should_define_in_comptime_main = true
            }
        }

        panic!("This should not happen - missing StackFrame in scope chain")
    }

    pub fn define_local(&mut self, name: String) -> StackFrameLocalRef {
        let local_ref = self.define_stack_frame_local();

        let block = match self.stack.last_mut() {
            Some(Scope::BlockScope(block)) => block,
            None => panic!("The scope stack is empty"),
            _ => panic!("The last scope in the stack should always be a block")
        };

        block.set_name(name, BlockNameRef::Local(local_ref));

        local_ref
    }

    pub fn define_comptime_main_local(&mut self, name: String) -> StackFrameLocalRef {
        let local_ref = self.define_comptime_main_stack_frame_local();

        let block = match self.stack.last_mut() {
            Some(Scope::BlockScope(block)) => block,
            None => panic!("The scope stack is empty"),
            _ => panic!("The last scope in the stack should always be a block")
        };

        block.set_name(name, BlockNameRef::Comptime((local_ref, None)));

        local_ref
    }

    pub fn access_local(&mut self, name: &str) -> Result<AccessNameRef, NameAccessError> {
        // By default, code is runtime, so we need to access comptime vals through exports.
        // However, If we pass through a ComptimePortal, then this will get changed to `false`.
        match self.access_name(name, true) {
            Ok(NameRef::Global(global_ref)) => Ok(AccessNameRef::Global(global_ref)),
            Ok(NameRef::ComptimeExport(export_ref, first_access)) => Ok(AccessNameRef::ComptimeExport(export_ref, first_access)),
            Ok(NameRef::ComptimeLocal(_)) => panic!("Got comptime local from call to access_name with export_comptime = true"),
            Ok(NameRef::Local(local_ref)) => Ok(AccessNameRef::Local(local_ref)),
            Err(error) => Err(error)
        }
    }

    fn access_name(&mut self, name: &str, export_comptime: bool) -> Result<NameRef, NameAccessError> {
        let mut i = self.stack.len() - 1;
        let mut export_comptime = export_comptime;
        let mut result = None;

        // Find the name in a block scope
        while i > 0 {
            match &mut self.stack[i] {
                Scope::RootScope(_) => {}
                Scope::ComptimeMainFrame(_) => {}
                Scope::StackFrame(_) => {}

                Scope::BlockScope(scope) => {
                    match scope.find_name(name) {
                        None => {},
                        Some(BlockNameRef::Local(local_ref)) => {
                            result = Some(NameRef::Local(local_ref));

                            break
                        },
                        Some(BlockNameRef::Comptime((local_ref, export_ref))) => {
                            if export_comptime {
                                if let Some(export_ref) = export_ref {
                                    result = Some(NameRef::ComptimeExport(export_ref, None));
                                } else {
                                    result = Some(NameRef::ComptimeLocal(local_ref))
                                }
                            } else {
                                result = Some(NameRef::ComptimeLocal(local_ref))
                            }

                            break
                        }
                    }
                },

                Scope::ComptimePortal(_) => export_comptime = false
            }

            i -= 1;
        }

        // i is the index of the block that we found the name in, we possibly need to export it
        if let Some(NameRef::ComptimeLocal(local_ref)) = result {
            if export_comptime {
                let export_ref = self.define_comptime_export();

                match &mut self.stack[i] {
                    Scope::BlockScope(scope) => scope.set_name(String::from(name), BlockNameRef::Comptime((local_ref, Some(export_ref)))),
                    _ => panic!("Logic error. Expected block scope")
                }

                result = Some(NameRef::ComptimeExport(export_ref, Some(local_ref)));
            }
        }

        if let Some(mut result) = result {
            i += 1;

            // Walk up the stack to process the result
            while i < self.stack.len() {
                match &mut self.stack[i] {
                    Scope::RootScope(_) => panic!("Not possible"),
                    Scope::ComptimeMainFrame(_) => panic!("Not possible"),
                    Scope::BlockScope(_) => {}

                    Scope::StackFrame(frame) => {
                        match result {
                            NameRef::Local(parent_ref) => {
                                result = NameRef::Local(frame.define_capture(parent_ref))
                            }
                            _ => {}
                        }
                    },

                    Scope::ComptimePortal(_) => {
                        match result {
                            NameRef::ComptimeExport(_, _) => panic!("This shouldn't happen"),
                            NameRef::Local(_) => return Err(NameAccessError::CannotReferenceRuntimeNameFromComptime),
                            NameRef::ComptimeLocal(local_ref) => result = NameRef::Local(local_ref),
                            _ => {}
                        }
                    }
                }

                i += 1;
            }

            return Ok(result)
        }

        Err(NameAccessError::NameNotFound)
    }
}

/// This is the same as NameRef from LexicalScope but without the ComptimeLocal, since that is
/// not possible to return from BlockScope::access_local
#[derive(Debug, PartialEq)]
pub enum AccessNameRef {
    /// The name is a global which can be loaded directly from the globals
    Global(GlobalRef),

    /// The name is a compile-time export which can be loaded from the rodata section
    ComptimeExport(ComptimeExportRef, Option<StackFrameLocalRef>),

    /// The name is defined in a parent stack frame
    Local(StackFrameLocalRef)
}

#[derive(Debug, PartialEq)]
enum NameRef {
    /// The name is a global which can be loaded directly from the globals
    Global(GlobalRef),

    /// The name is a compile-time export which can be loaded from the rodata section
    /// The second value is set only the first time - when the "set" instruction for the comptime
    /// export needs to be generated initially. On latter accesses it will be None
    ComptimeExport(ComptimeExportRef, Option<StackFrameLocalRef>),

    /// The name is defined in a parent stack frame. The stack frame is only present at compile time
    ComptimeLocal(StackFrameLocalRef),

    /// The name is defined in a parent stack frame
    Local(StackFrameLocalRef)
}

#[derive(Debug, PartialEq)]
pub enum NameAccessError {
    NameNotFound,
    CannotReferenceRuntimeNameFromComptime
}