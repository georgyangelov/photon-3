use crate::compiler::lexical_scope::LexicalScope::{BlockScope, FnScope, RootScope};
use crate::compiler::mir::{LocalSlotRef};

pub enum LexicalScope<'a> {
    RootScope { globals: Vec<Global> },

    FnScope { parent: &'a mut LexicalScope<'a>, stack_slots: Vec<Local>, captures: Vec<Capture> },

    BlockScope { parent: &'a mut LexicalScope<'a>, locals: Vec<(String, LocalSlotRef)> }
}

// pub struct LexicalScope<'a> {
//     parent: Option<&'a mut LexicalScope<'a>>,
//
//     locals: Vec<Local>
// }

struct Global {
    name: String
    // typ: MIRType,
}

struct Local {
    name: String,
    // typ: MIRType,
}

#[derive(Copy)]
pub enum SlotRef {
    Global(GlobalSlotRef),
    Local(LocalSlotRef)
}

#[derive(Copy)]
pub struct LocalSlotRef {
    i: usize
}

#[derive(Copy)]
pub struct GlobalSlotRef {
    i: usize
}

#[derive(Copy)]
pub struct Capture {
    pub from: LocalSlotRef,
    pub to: LocalSlotRef
}

impl <'a> LexicalScope<'a> {
    pub fn new_root() -> Self {
        RootScope { globals: Vec::new() }
    }

    pub fn new_child_block(&mut self) -> Self {
        BlockScope { parent: self, locals: Vec::new() }
    }

    pub fn new_child_fn(&mut self, params: Vec<String>) -> Self {
        FnScope {
            parent: self,
            stack_slots: params.into_iter().map(|name| Local { name }).collect(),
            captures: Vec::new()
        }
    }

    pub fn define_name(&mut self, name: String) -> LocalSlotRef {
        match self {
            RootScope { .. } => panic!("Cannot define names in the global scope"),
            FnScope { stack_slots, .. } => {
                let i = stack_slots.len();

                stack_slots.push(Local { name });

                LocalSlotRef { i }
            }
            BlockScope { parent, locals } => {
                let slot_ref = parent.define_stack_slot(name.clone());

                locals.push((name, slot_ref));

                slot_ref
            }
        }
    }

    fn define_stack_slot(&mut self, name: String) -> LocalSlotRef {
        match self {
            RootScope { .. } => panic!("Cannot define names in the global scope"),
            FnScope { .. } => self.define_name(name),
            BlockScope { parent, .. } => parent.define_stack_slot(name)
        }
    }

    pub fn access_name(&mut self, name: &str) -> Option<SlotRef> {
        match self {
            RootScope { globals } => {
                for (i, global) in globals.iter().enumerate() {
                    if global.name == name {
                        return Some(SlotRef::Global(GlobalSlotRef { i }))
                    }
                }

                None
            }

            FnScope { parent, stack_slots, captures } => {
                for (i, local) in stack_slots.iter().enumerate() {
                    if local.name == name {
                        return Some(SlotRef::Local(LocalSlotRef { i }))
                    }
                }

                let parent_slot_ref = match parent.access_name(name) {
                    None => return None,
                    Some(global_ref @ SlotRef::Global(_)) => return Some(global_ref),
                    Some(SlotRef::Local(local_ref)) => local_ref
                };

                let captured_slot_ref = self.define_name(String::from(name));

                captures.push(Capture { from: parent_slot_ref, to: captured_slot_ref });

                Some(SlotRef::Local(captured_slot_ref))
            }

            BlockScope { parent, locals } => {
                for (local_name, local_ref) in locals {
                    if local_name == name {
                        return Some(SlotRef::Local(*local_ref))
                    }
                }

                parent.access_name(name)
            }
        }
    }
}
