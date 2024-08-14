/// This indicates a switch to compile-time code. It acts as a portal to the ComptimeMainStackFrame
/// above it. If a new variable is defined by any block inside, it will be defined as a comptime
/// local in the current stack frame.
pub struct ComptimePortal {}

impl ComptimePortal {
    pub fn new() -> Self {
        ComptimePortal {}
    }
}