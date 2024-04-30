pub struct RootScope {
    // TODO: Separate globals for compile-time code and for run-time code.
    //       Actually it's probably better to annotate the globals so that we can provide
    //       useful error messages like "Cannot use fs in a compile-time context"
    globals: Vec<Global>,

    compile_time_slot: Vec<CompileTimeSlot>

    // statics: Vec<Static>
}

pub trait LexicalScope {
    fn define_compile_time_slot(&mut self, name: Option<String>) -> CompileTimeSlotRef;
    fn define_stack_slot(&mut self, name: Option<String>) -> LocalSlotRef;
    fn access_name(&mut self, name: &str) -> Option<SlotRef>;
}

impl RootScope {
    pub fn new() -> Self {
        RootScope { globals: Vec::new(), compile_time_slot: Vec::new() }
    }
}

impl LexicalScope for RootScope {
    fn define_compile_time_slot(&mut self, name: Option<String>) -> CompileTimeSlotRef {
        let i = self.compile_time_slot.len();

        self.compile_time_slot.push(CompileTimeSlot { name });

        CompileTimeSlotRef { i }
    }

    fn define_stack_slot(&mut self, _: Option<String>) -> LocalSlotRef {
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
    pub fn new(parent: &'a mut dyn LexicalScope, params: Vec<String>) -> Self {
        Self {
            parent,
            stack_slots: params.into_iter().map(|name| Local { name: Some(name) }).collect(),
            captures: Vec::new()
        }
    }
}

impl <'a> LexicalScope for FnScope<'a> {
    fn define_compile_time_slot(&mut self, name: Option<String>) -> CompileTimeSlotRef {
        self.parent.define_compile_time_slot(name)
    }

    fn define_stack_slot(&mut self, name: Option<String>) -> LocalSlotRef {
        let i = self.stack_slots.len();

        self.stack_slots.push(Local { name });

        LocalSlotRef { i }
    }

    fn access_name(&mut self, name: &str) -> Option<SlotRef> {
        for (i, local) in self.stack_slots.iter().enumerate() {
            if let Some(local_name) = &local.name {
                if local_name == name {
                    return Some(SlotRef::Local(LocalSlotRef { i }))
                }
            }
        }

        let parent_slot_ref = match self.parent.access_name(name) {
            None => return None,
            Some(SlotRef::Local(local_ref)) => local_ref,
            Some(other_ref @ _) => return Some(other_ref),
        };

        let captured_slot_ref = self.define_stack_slot(Some(String::from(name)));

        self.captures.push(Capture { from: parent_slot_ref, to: captured_slot_ref });

        Some(SlotRef::Local(captured_slot_ref))
    }
}

pub struct BlockScope<'a> {
    parent: &'a mut dyn LexicalScope,
    locals: Vec<(Option<String>, SlotRef)>
}

impl <'a> BlockScope<'a> {
    pub fn new(parent: &'a mut dyn LexicalScope) -> Self {
        BlockScope { parent, locals: Vec::new() }
    }

    pub fn define_name(&mut self, name: Option<String>) -> LocalSlotRef {
        let slot_ref = self.parent.define_stack_slot(name.clone());

        self.locals.push((name, SlotRef::Local(slot_ref)));

        slot_ref
    }

    pub fn define_compile_time_local_name(
        &mut self,
        name: String,

        // This is the local ref in the compile-time code. We will copy this to a
        // compile-time slot once it's used in run-time code.
        // Otherwise, it's only local to the compile-time code.
        compile_time_local_scope_ref: LocalSlotRef
    ) {
        self.locals.push((Some(name), SlotRef::CompileTimeLocal(compile_time_local_scope_ref)));
    }

    pub fn define_compile_time_export_name(
        &mut self,
        name: String
    ) -> CompileTimeSlotRef {
        let slot_ref = self.parent.define_compile_time_slot(Some(name.clone()));

        let mut index = None;
        for (i, (slot_name, _)) in self.locals.iter().enumerate() {
            if let Some(slot_name) = slot_name {
                if slot_name == &name {
                    index = Some(i);
                }
            }
        }

        if let Some(index) = index {
            self.locals[index] = (Some(name), SlotRef::CompileTime(slot_ref));
        } else {
            self.locals.push((Some(name), SlotRef::CompileTime(slot_ref)));
        }

        slot_ref
    }
}

impl <'a> LexicalScope for BlockScope<'a> {
    fn define_compile_time_slot(&mut self, name: Option<String>) -> CompileTimeSlotRef {
        self.parent.define_compile_time_slot(name)
    }

    fn define_stack_slot(&mut self, name: Option<String>) -> LocalSlotRef {
        self.parent.define_stack_slot(name)
    }

    fn access_name(&mut self, name: &str) -> Option<SlotRef> {
        for (local_name, slot_ref) in &self.locals {
            if let Some(local_name) = local_name {
                if local_name == name {
                    return Some(*slot_ref)
                }
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
    name: Option<String>
    // typ: MIRType,
}

pub struct CompileTimeSlot {
    name: Option<String>
}

#[derive(Copy, Clone)]
pub struct CompileTimeSlotRef {
    i: usize
}

#[derive(Copy, Clone)]
pub enum SlotRef {
    CompileTime(CompileTimeSlotRef),
    CompileTimeLocal(LocalSlotRef),
    Global(GlobalSlotRef),
    Local(LocalSlotRef)
}

#[derive(Copy, Clone)]
pub struct LocalSlotRef {
    i: usize
}

#[derive(Copy, Clone)]
pub struct GlobalSlotRef {
    i: usize
}

#[derive(Copy, Clone)]
pub struct Capture {
    pub from: LocalSlotRef,
    pub to: LocalSlotRef
}
