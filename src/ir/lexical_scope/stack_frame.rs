use crate::ir::{CaptureFrom, CaptureRef, LocalRef, ParamRef};

/// A function/closure scope - it has locals and can reference variables from parent scopes by
/// capturing them.
pub struct StackFrame {
    /// The captured values from parent scopes
    pub captures: Vec<Capture>,

    /// Parameters
    pub params: Vec<Param>,

    /// The local stack frame slots
    pub locals: Vec<StackFrameLocal>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Capture {
    /// The name it's referenced by
    pub name: String,

    /// The local to capture from the parent stack frame
    pub from: CaptureFrom,

    pub comptime: bool
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// The name it's referenced by
    pub name: String,

    /// Is this a comptime param?
    pub comptime: bool
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackFrameLocal {
    // TODO: Include an optional name, for debugging purposes

    pub comptime: bool
}

impl StackFrame {
    pub fn new(params: Vec<Param>) -> Self {
        Self {
            captures: Vec::new(),
            params,
            locals: Vec::new()
        }
    }

    pub fn define_stack_frame_local(&mut self, comptime: bool) -> LocalRef {
        let i = self.locals.len();

        self.locals.push(StackFrameLocal { comptime });

        LocalRef { i, comptime }
    }

    pub fn define_capture(&mut self, from: CaptureFrom, name: String) -> CaptureRef {
        let i = self.captures.len();

        let comptime = match from {
            CaptureFrom::Capture(capture) => capture.comptime,
            CaptureFrom::Param(param) => param.comptime,
            CaptureFrom::Local(local) => local.comptime
        };

        self.captures.push(Capture { name, from, comptime });

        CaptureRef { i, comptime }
    }

    pub fn find_param_or_capture(&self, name: &str) -> Option<ParamOrCapture> {
        for (i, param) in self.params.iter().enumerate() {
            if param.name == name {
                return Some(ParamOrCapture::Param(ParamRef { i, comptime: param.comptime }))
            }
        }

        for (i, capture) in self.captures.iter().enumerate() {
            if capture.name == name {
                return Some(ParamOrCapture::Capture(CaptureRef { i, comptime: capture.comptime }))
            }
        }

        None
    }
}

pub enum ParamOrCapture {
    Param(ParamRef),
    Capture(CaptureRef)
}