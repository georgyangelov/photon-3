use crate::mir::{ComptimeExportRef, GlobalRef};
use crate::mir::lexical_scope::*;

pub struct RootScope {
    globals: Vec<String>,

    pub comptime_exports: Vec<ComptimeExportSlot>,
}

pub struct ComptimeExportSlot {}

impl RootScope {
    pub fn new(globals: Vec<String>) -> Self {
        Self {
            globals,
            comptime_exports: Vec::new()
        }
    }

    pub fn define_comptime_export(&mut self) -> ComptimeExportRef {
        let i = self.comptime_exports.len();

        self.comptime_exports.push(ComptimeExportSlot {});

        ComptimeExportRef { i }
    }

    pub fn find_global(&mut self, name: &str) -> Option<GlobalRef> {
        for (i, global_name) in self.globals.iter().enumerate() {
            if global_name == name {
                return Some(GlobalRef { i })
            }
        }

        None
    }
}
