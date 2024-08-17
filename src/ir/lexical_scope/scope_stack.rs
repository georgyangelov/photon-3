use crate::ir::lexical_scope::*;
use crate::ir::{CaptureFrom, CaptureRef, GlobalRef, LocalRef, ParamRef};

pub struct ScopeStack {
    stack: Vec<Scope>
}

pub enum Scope {
    RootScope(RootScope),
    ComptimePortal(ComptimePortal),
    StackFrame(StackFrame),
    BlockScope(BlockScope)
}

impl ScopeStack {
    pub fn new(root: RootScope) -> Self {
        ScopeStack {
            stack: vec![
                Scope::RootScope(root)
            ]
        }
    }

    pub fn consume_root(mut self) -> RootScope {
        let root = self.pop_root();

        root
    }

    pub fn push_block(&mut self) {
        let scope = BlockScope::new();

        self.stack.push(Scope::BlockScope(scope))
    }

    pub fn push_stack_frame(&mut self, params: Vec<Param>) {
        let scope = StackFrame::new(params);

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

    // TODO: Specify `comptime` inside of the LocalRef as well
    pub fn define_stack_frame_local(&mut self, mut comptime: bool) -> LocalRef {
        for i in (0..self.stack.len()).rev() {
            match &mut self.stack[i] {
                Scope::RootScope(_) => {},
                Scope::StackFrame(frame) => return frame.define_stack_frame_local(comptime),
                Scope::BlockScope(_) => {}
                Scope::ComptimePortal(_) => comptime = true
            }
        }

        panic!("This should not happen - missing StackFrame in scope chain")
    }

    pub fn define_local(&mut self, name: String, comptime: bool) -> LocalRef {
        let local_ref = self.define_stack_frame_local(comptime);

        let block = match self.stack.last_mut() {
            Some(Scope::BlockScope(block)) => block,
            None => panic!("The scope stack is empty"),
            _ => panic!("The last scope in the stack should always be a block")
        };

        block.set_name(name, local_ref);

        local_ref
    }

    pub fn lookup(&mut self, name: &str) -> Result<NameRef, NameAccessError> {
        let mut i = self.stack.len() - 1;
        let mut result = None;

        // Find the name
        loop {
            match &mut self.stack[i] {
                Scope::RootScope(scope) => {
                    match scope.find_global(name) {
                        None => {}
                        Some(global_ref) => {
                            result = Some(NameRef::Global(global_ref));
                            break
                        }
                    }
                }

                Scope::StackFrame(scope) => {
                    match scope.find_param_or_capture(name) {
                        None => {}
                        Some(ParamOrCapture::Param(param_ref)) => {
                            result = Some(NameRef::Param(param_ref));
                            break
                        }
                        Some(ParamOrCapture::Capture(capture_ref)) => {
                            result = Some(NameRef::Capture(capture_ref));
                            break
                        }
                    }
                }

                Scope::BlockScope(scope) => {
                    match scope.find_name(name) {
                        None => {}
                        Some(local_ref) => {
                            result = Some(NameRef::Local(local_ref));
                            break
                        }
                    }

                    match scope.find_name(name) {
                        None => {},
                        Some(local_ref) => {
                            result = Some(NameRef::Local(local_ref));
                            break
                        }
                    }
                }

                Scope::ComptimePortal(_) => {}
            }

            if i == 0 {
                break
            }

            i -= 1;
        }

        if let Some(mut result) = result {
            i += 1;

            let comptime = match result {
                NameRef::Global(global_ref) => global_ref.comptime,
                NameRef::Capture(capture_ref) => capture_ref.comptime,
                NameRef::Param(param_ref) => param_ref.comptime,
                NameRef::Local(local_ref) => local_ref.comptime
            };

            // Walk down the stack to process the result
            while i < self.stack.len() {
                match &mut self.stack[i] {
                    Scope::RootScope(_) => panic!("Not possible"),
                    Scope::BlockScope(_) => {}

                    Scope::StackFrame(frame) => {
                        let capture_from = match result {
                            NameRef::Capture(capture_ref) => Some(CaptureFrom::Capture(capture_ref)),
                            NameRef::Param(param_ref) => Some(CaptureFrom::Param(param_ref)),
                            NameRef::Local(local_ref) => Some(CaptureFrom::Local(local_ref)),
                            _ => None
                        };

                        match capture_from {
                            None => {}
                            Some(capture_from) => {
                                let capture_ref = frame.define_capture(capture_from, String::from(name));

                                result = NameRef::Capture(capture_ref);
                            }
                        }
                    },

                    Scope::ComptimePortal(_) => {
                        match result {
                            NameRef::Capture(_) | NameRef::Param(_) | NameRef::Local(_) => {
                                if !comptime {
                                    return Err(NameAccessError::CannotReferenceRuntimeNameFromComptime)
                                }
                            }
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

#[derive(Debug, PartialEq)]
pub enum NameRef {
    /// The name is a global which can be loaded directly from the globals
    Global(GlobalRef),

    /// The name is accessed from a parent scope in a closure
    Capture(CaptureRef),

    /// The name is a function parameter of the currently-executing function
    /// If we're in a closure, and we're referencing a parent function's arguments, then that would
    /// be a [NameRef::Capture], not [NameRef::Param].
    Param(ParamRef),

    /// The name is defined in a parent stack frame
    Local(LocalRef)
}

#[derive(Debug, PartialEq)]
pub enum NameAccessError {
    NameNotFound,
    CannotReferenceRuntimeNameFromComptime
}