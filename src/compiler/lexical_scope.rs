use crate::compiler::mir::{LocalSlotRef};

pub enum LexicalScope<'a> {
    Root(&'a mut RootScope),
    Fn(&'a mut FnScope<'a>),
    Block(&'a mut BlockScope<'a>)
}

pub struct RootScope {
    globals: Vec<Global>,
    // statics: Vec<Static>
}

impl RootScope {
    fn new() -> Self {
        RootScope { globals: Vec::new() }
    }
}

pub struct FnScope<'a> {
    parent: &'a mut LexicalScope<'a>,
    stack_slots: Vec<Local>,
    captures: Vec<Capture>
}

impl <'a> FnScope<'a> {
    pub fn define_name(&mut self, name: String) -> LocalSlotRef {
        let i = self.stack_slots.len();

        self.stack_slots.push(Local { name });

        LocalSlotRef { i }
    }
}

pub struct BlockScope<'a> {
    parent: &'a mut LexicalScope<'a>,
    locals: Vec<(String, LocalSlotRef)>
}

impl <'a> BlockScope<'a> {
    pub fn define_name(&mut self, name: String) -> LocalSlotRef {
        let slot_ref = self.parent.define_stack_slot(name.clone());

        self.locals.push((name, slot_ref));

        slot_ref
    }
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

struct Static {
    name: String
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
    pub fn new_child_block(&mut self) -> BlockScope {
        BlockScope { parent: self, locals: Vec::new() }
    }

    pub fn new_child_fn(&mut self, params: Vec<String>) -> FnScope {
        FnScope {
            parent: self,
            stack_slots: params.into_iter().map(|name| Local { name }).collect(),
            captures: Vec::new()
        }
    }

    pub fn define_name(&mut self, name: String) -> LocalSlotRef {
        match self {
            LexicalScope::Root(_) => panic!("Cannot define names in the global scope"),
            LexicalScope::Fn(scope) => scope.define_name(name),
            LexicalScope::Block(scope) => scope.define_name(name)
        }
    }

    fn define_stack_slot(&mut self, name: String) -> LocalSlotRef {
        match self {
            LexicalScope::Root(_) => panic!("Cannot define names in the global scope"),
            LexicalScope::Fn(scope) => scope.define_name(name),
            LexicalScope::Block(scope) => scope.parent.define_stack_slot(name)
        }
    }

    pub fn access_name(&mut self, name: &str) -> Option<SlotRef> {
        match self {
            LexicalScope::Root(scope) => {
                for (i, global) in scope.globals.iter().enumerate() {
                    if global.name == name {
                        return Some(SlotRef::Global(GlobalSlotRef { i }))
                    }
                }

                None
            }

            LexicalScope::Fn(scope) => {
                for (i, local) in scope.stack_slots.iter().enumerate() {
                    if local.name == name {
                        return Some(SlotRef::Local(LocalSlotRef { i }))
                    }
                }

                let parent_slot_ref = match scope.parent.access_name(name) {
                    None => return None,
                    Some(global_ref @ SlotRef::Global(_)) => return Some(global_ref),
                    Some(SlotRef::Local(local_ref)) => local_ref
                };

                let captured_slot_ref = self.define_name(String::from(name));

                scope.captures.push(Capture { from: parent_slot_ref, to: captured_slot_ref });

                Some(SlotRef::Local(captured_slot_ref))
            }

            LexicalScope::Block(scope) => {
                for (local_name, local_ref) in scope.locals {
                    if local_name == name {
                        return Some(SlotRef::Local(*local_ref))
                    }
                }

                scope.parent.access_name(name)
            }
        }
    }
}
