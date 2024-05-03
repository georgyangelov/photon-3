mod root_scope;
mod comptime_main;
mod stack_frame;
mod block_scope;
mod comptime_portal;
mod scope_stack;

pub use root_scope::*;
pub use comptime_main::*;
pub use stack_frame::*;
pub use block_scope::*;
pub use comptime_portal::*;
pub use scope_stack::*;

struct StackFrameLocal {
    // TODO: Include an optional name, for debugging purposes
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GlobalRef { i: usize }

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ComptimeExportRef { i: usize }

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StackFrameLocalRef { i: usize }