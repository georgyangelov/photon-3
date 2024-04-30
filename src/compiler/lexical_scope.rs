use crate::compiler::mir::{LocalSlotRef};

pub struct RootScope {
    // TODO: Separate globals for compile-time code and for run-time code
    globals: Vec<Global>,

    compile_time_slot: Vec<CompileTimeSlot>

    // statics: Vec<Static>
}

pub trait LexicalScope {
    fn define_compile_time_slot(&mut self, name: Option<String>) -> CompileTimeSlotRef;
    fn define_stack_slot(&mut self, name: String) -> LocalSlotRef;
    fn access_name(&mut self, name: &str) -> Option<SlotRef>;
}

impl RootScope {
    fn new() -> Self {
        RootScope { globals: Vec::new(), compile_time_slot: Vec::new() }
    }
}

impl LexicalScope for RootScope {
    fn define_compile_time_slot(&mut self, name: Option<String>) -> CompileTimeSlotRef {
        let i = self.compile_time_slot.len();

        self.compile_time_slot.push(CompileTimeSlot { name });

        CompileTimeSlotRef { i }
    }

    fn define_stack_slot(&mut self, name: String) -> LocalSlotRef {
        panic!("Cannot define new stack slots in the global scope")
    }

    fn access_name(&mut self, name: &str) -> Option<SlotRef> {
        for (i, global) in self.globals.iter().enumerate() {
            if global.name == name {
                return Some(SlotRef::Global(GlobalSlotRef { i }))
            }
        }

        None
    }
}

// pub enum ParentScope<'a> {
//     Root(&'a mut RootScope),
//     Fn(&'a mut FnScope<'a>),
//     Block(&'a mut BlockScope<'a>)
// }

pub struct FnScope<'a> {
    parent: &'a mut dyn LexicalScope,
    stack_slots: Vec<Local>,
    captures: Vec<Capture>
}

impl <'a> FnScope<'a> {
    pub fn new(parent: &mut dyn LexicalScope, params: Vec<String>) -> Self {
        FnScope {
            parent,
            stack_slots: params.into_iter().map(|name| Local { name }).collect(),
            captures: Vec::new()
        }
    }
}

impl <'a> LexicalScope for FnScope<'a> {
    fn define_compile_time_slot(&mut self, name: Option<String>) -> CompileTimeSlotRef {
        self.parent.define_compile_time_slot(name)
    }

    fn define_stack_slot(&mut self, name: String) -> LocalSlotRef {
        let i = self.stack_slots.len();

        self.stack_slots.push(Local { name });

        LocalSlotRef { i }
    }

    fn access_name(&mut self, name: &str) -> Option<SlotRef> {
        for (i, local) in self.stack_slots.iter().enumerate() {
            if local.name == name {
                return Some(SlotRef::Local(LocalSlotRef { i }))
            }
        }

        let parent_slot_ref = match self.parent.access_name(name) {
            None => return None,
            Some(global_ref @ SlotRef::Global(_)) => return Some(global_ref),
            Some(SlotRef::Local(local_ref)) => local_ref
        };

        let captured_slot_ref = self.define_name(String::from(name));

        self.captures.push(Capture { from: parent_slot_ref, to: captured_slot_ref });

        Some(SlotRef::Local(captured_slot_ref))
    }
}

pub struct BlockScope<'a> {
    parent: &'a mut dyn LexicalScope<'a>,
    locals: Vec<(String, SlotRef)>
}

impl <'a> BlockScope<'a> {
    pub fn new(parent: &mut dyn LexicalScope) -> Self {
        BlockScope { parent, locals: Vec::new() }
    }

    pub fn define_name(&mut self, name: String) -> LocalSlotRef {
        let slot_ref = self.parent.define_stack_slot(name.clone());

        self.locals.push((name, SlotRef::Local(slot_ref)));

        slot_ref
    }

    pub fn define_compile_time_name(&mut self, name: String) -> CompileTimeSlotRef {
        let slot_ref = self.parent.define_compile_time_slot(Some(name.clone()));

        self.locals.push((name, SlotRef::CompileTime(slot_ref)));

        slot_ref
    }
}

impl <'a> LexicalScope for BlockScope<'a> {
    fn define_compile_time_slot(&mut self, name: Option<String>) -> CompileTimeSlotRef {
        self.parent.define_compile_time_slot(name)
    }

    fn define_stack_slot(&mut self, name: String) -> LocalSlotRef {
        self.parent.define_stack_slot(name)
    }

    fn access_name(&mut self, name: &str) -> Option<SlotRef> {
        for (local_name, slot_ref) in &self.locals {
            if local_name == name {
                return Some(*slot_ref)
            }
        }

        self.parent.access_name(name)
    }
}

struct Global {
    name: String
    // typ: MIRType,
}

struct Static {
    name: String
}

struct Local {
    name: String
    // typ: MIRType,
}

pub struct CompileTimeSlot {
    name: Option<String>
}

#[derive(Copy)]
pub struct CompileTimeSlotRef {
    i: usize
}

#[derive(Copy)]
pub enum SlotRef {
    CompileTime(CompileTimeSlotRef),
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
