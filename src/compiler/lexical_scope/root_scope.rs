use crate::compiler::lexical_scope::*;

pub struct RootScope {
    // runtime_globals: Vec<Global>,
    // comptime_globals: Vec<Global>,

    comptime_exports: Vec<ComptimeExportSlot>,
}

struct ComptimeExportSlot {}

struct Global {
    name: String
}

impl RootScope {
    pub fn new() -> Self {
        Self {
            // runtime_globals: Vec::new(),
            // comptime_globals: Vec::new(),
            comptime_exports: Vec::new()
        }
    }

    pub fn define_comptime_export(&mut self) -> ComptimeExportRef {
        let i = self.comptime_exports.len();

        self.comptime_exports.push(ComptimeExportSlot {});

        ComptimeExportRef { i }
    }
}
