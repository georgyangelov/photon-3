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

pub struct StackFrameLocal {
    // TODO: Include an optional name, for debugging purposes
}
