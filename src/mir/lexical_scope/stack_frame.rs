use crate::mir::lexical_scope::*;

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
    pub from: CaptureFrom
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// The name it's referenced by
    pub name: String
}

impl StackFrame {
    pub fn new(params: Vec<String>) -> Self {
        Self {
            captures: Vec::new(),
            params: params.into_iter().map(|name| Param { name }).collect(),
            locals: Vec::new()
        }
    }

    pub fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        let i = self.locals.len();

        self.locals.push(StackFrameLocal {});

        StackFrameLocalRef { i }
    }

    // pub fn define_param(&mut self, name: String) -> ParamRef {
    //     let i = self.params.len();
    //
    //     self.params.push(Param { name });
    //
    //     ParamRef { i }
    // }

    pub fn define_capture(&mut self, from: CaptureFrom, name: String) -> CaptureRef {
        let i = self.captures.len();

        self.captures.push(Capture { name, from });

        CaptureRef { i }
    }

    pub fn find_param_or_capture(&self, name: &str) -> Option<ParamOrCapture> {
        for (i, param) in self.params.iter().enumerate() {
            if param.name == name {
                return Some(ParamOrCapture::Param(ParamRef { i }))
            }
        }

        for (i, capture) in self.captures.iter().enumerate() {
            if capture.name == name {
                return Some(ParamOrCapture::Capture(CaptureRef { i }))
            }
        }

        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureFrom {
    Capture(CaptureRef),
    Param(ParamRef),
    Local(StackFrameLocalRef)
}

pub enum ParamOrCapture {
    Param(ParamRef),
    Capture(CaptureRef)
}