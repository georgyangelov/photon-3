use crate::ir::{Global, GlobalRef};

pub struct RootScope {
    globals: Vec<Global>,

    pub comptime_exports: Vec<ComptimeExportSlot>,
}

pub struct ComptimeExportSlot {}

impl RootScope {
    pub fn new(globals: Vec<Global>) -> Self {
        Self {
            globals,
            comptime_exports: Vec::new()
        }
    }

    pub fn find_global(&mut self, name: &str) -> Option<GlobalRef> {
        for (i, global) in self.globals.iter().enumerate() {
            if global.name == name {
                return Some(GlobalRef { i, comptime: global.comptime })
            }
        }

        None
    }
}
