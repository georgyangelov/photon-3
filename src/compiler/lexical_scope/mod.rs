mod root_scope;
mod comptime_main;
mod stack_frame;
mod block_scope;
mod comptime_portal;

pub use root_scope::*;
pub use comptime_main::*;
pub use stack_frame::*;
pub use block_scope::*;
pub use comptime_portal::*;

pub trait LexicalScope {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef;
    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef;
    fn define_comptime_export(&mut self) -> ComptimeExportRef;

    /// export_comptime - whether comptime stack frames need to be accessed through comptime exports
    fn access_name(&mut self, name: &str, export_comptime: bool) -> Result<NameRef, NameAccessError>;
}

#[derive(Debug, PartialEq)]
pub enum NameRef {
    /// The name is a global which can be loaded directly from the globals
    Global(GlobalRef),

    /// The name is a compile-time export which can be loaded from the rodata section
    ComptimeExport(ComptimeExportRef),

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

struct StackFrameLocal {
    // TODO: Include an optional name, for debugging purposes
}

#[derive(Debug, PartialEq)]
pub struct Capture {
    /// The local to capture from the parent stack frame
    pub from: StackFrameLocalRef,

    /// The local of the child stack frame to put the captured value in
    pub to: StackFrameLocalRef
}

struct ComptimeExportSlot {}





struct Global {
    name: String
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GlobalRef { i: usize }

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ComptimeExportRef { i: usize }

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StackFrameLocalRef { i: usize }