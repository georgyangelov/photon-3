use crate::compiler::lexical_scope::*;

/// A function/closure scope - it has locals and can reference variables from parent scopes by
/// capturing them.
pub struct StackFrame {
    /// The captured values from parent scopes
    pub captures: Vec<Capture>,

    /// The local stack frame slots
    pub locals: Vec<StackFrameLocal>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Capture {
    /// The local to capture from the parent stack frame
    pub from: StackFrameLocalRef,

    /// The local of the child stack frame to put the captured value in
    pub to: StackFrameLocalRef
}

impl StackFrame {
    pub fn new() -> Self {
        Self {
            captures: Vec::new(),
            locals: Vec::new()
        }
    }

    pub fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        let i = self.locals.len();

        self.locals.push(StackFrameLocal {});

        StackFrameLocalRef { i }
    }

    pub fn define_capture(&mut self, parent_local_ref: StackFrameLocalRef) -> StackFrameLocalRef {
        let child_local_ref = self.define_stack_frame_local();

        self.captures.push(Capture {
            from: parent_local_ref,
            to: child_local_ref
        });

        child_local_ref
    }
}
