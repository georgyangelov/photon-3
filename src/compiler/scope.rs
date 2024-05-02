pub trait Scope {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef;
    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef;
    fn define_comptime_export(&mut self) -> ComptimeExportRef;

    /// export_comptime - whether comptime stack frames need to be accessed through comptime exports
    fn access_name(&mut self, name: &str, export_comptime: bool) -> Result<NameRef, NameAccessError>;
}

// pub enum ComptimeNameRef {
//     /// The name is a global which can be loaded directly from the globals
//     Global(GlobalRef),
//
//     /// The name is defined in a parent stack frame
//     Local(StackFrameLocalRef)
// }

#[derive(Debug, PartialEq)]
pub enum NameRef {
    /// The name is a global which can be loaded directly from the globals
    Global(GlobalRef),

    /// The name is a compile-time export which can be loaded from the rodata section
    ComptimeExport(ComptimeExportRef),

    /// The name is defined in a parent stack frame. The stack frame is only present at compile time
    ComptimeLocal(StackFrameLocalRef),

    /// The name is defined in a parent stack frame
    Local(StackFrameLocalRef)
}

#[derive(Debug, PartialEq)]
pub enum NameAccessError {
    NameNotFound,
    CannotReferenceRuntimeNameFromComptime
}




pub struct RootScope {
    // runtime_globals: Vec<Global>,
    // comptime_globals: Vec<Global>,

    comptime_exports: Vec<ComptimeExportSlot>,
}

impl RootScope {
    pub fn new() -> Self {
        Self {
            // runtime_globals: Vec::new(),
            // comptime_globals: Vec::new(),
            comptime_exports: Vec::new()
        }
    }
}

impl Scope for RootScope {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        panic!("This should not happen - missing ComptimeMainStackFrame in scope chain")
    }

    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        panic!("This should not happen - missing StackFrame in scope chain")
    }

    fn define_comptime_export(&mut self) -> ComptimeExportRef {
        let i = self.comptime_exports.len();

        self.comptime_exports.push(ComptimeExportSlot {});

        ComptimeExportRef { i }
    }

    fn access_name(&mut self, name: &str, export_comptime: bool) -> Result<NameRef, NameAccessError> {
        // todo!("Find locals depending on if we're in comptime or not")
        Err(NameAccessError::NameNotFound)
    }
}



/// The compile-time main function. There is only one such scope per-module and it executes the
/// compile-time code.
///
/// It's also the only one that can export data from the compile-time code to the run-time code.
/// It does that by allocating "comptime export slots" on the RootScope, then copying locals from
/// the stack frame into the export slot (usually global memory), which is later included in the
/// runtime binary as static data and can be referenced from there.
pub struct ComptimeMainStackFrame<'a> {
    parent: &'a mut RootScope,

    /// The local stack frame slots - these would only be used during comptime run
    locals: Vec<StackFrameLocal>,

    /// Tracks local slots which need to be exported
    exports: Vec<(StackFrameLocalRef, ComptimeExportRef)>
}

impl <'a> ComptimeMainStackFrame<'a> {
    pub fn new(parent: &'a mut RootScope) -> Self {
        Self {
            parent,
            locals: Vec::new(),
            exports: Vec::new()
        }
    }
}

impl <'a> Scope for ComptimeMainStackFrame<'a> {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.define_stack_frame_local()
    }

    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        let i = self.locals.len();

        self.locals.push(StackFrameLocal {});

        StackFrameLocalRef { i }
    }

    fn define_comptime_export(&mut self) -> ComptimeExportRef {
        self.parent.define_comptime_export()
    }

    fn access_name(&mut self, name: &str, export_comptime: bool) -> Result<NameRef, NameAccessError> {
        self.parent.access_name(name, export_comptime)
    }
}




/// A function/closure scope - it has locals and can reference variables from parent scopes by
/// capturing them.
pub struct StackFrame<'a> {
    parent: &'a mut dyn Scope,

    /// The captured values from parent scopes
    pub captures: Vec<Capture>,

    /// The local stack frame slots
    locals: Vec<StackFrameLocal>,
}

impl <'a> StackFrame<'a> {
    pub fn new(parent: &'a mut dyn Scope) -> Self {
        Self {
            parent,
            captures: Vec::new(),
            locals: Vec::new()
        }
    }
}

impl <'a> Scope for StackFrame<'a> {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.parent.define_comptime_main_stack_frame_local()
    }

    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        let i = self.locals.len();

        self.locals.push(StackFrameLocal {});

        StackFrameLocalRef { i }
    }

    fn define_comptime_export(&mut self) -> ComptimeExportRef {
        self.parent.define_comptime_export()
    }

    fn access_name(&mut self, name: &str, export_comptime: bool) -> Result<NameRef, NameAccessError> {
        match self.parent.access_name(name, export_comptime)? {
            NameRef::Local(parent_local_ref) => {
                let child_local_ref = self.define_stack_frame_local();

                self.captures.push(Capture {
                    from: parent_local_ref,
                    to: child_local_ref
                });

                Ok(NameRef::Local(child_local_ref))
            },

            comptime_ref @ NameRef::ComptimeExport(_) => Ok(comptime_ref),
            local_ref @ NameRef::ComptimeLocal(_) => Ok(local_ref),
            global_ref @ NameRef::Global(_) => Ok(global_ref),
        }
    }
}




/// A simple block scope - it defines new variables in its closest parent stack frame and ensures
/// the defined name is only accessible by the children of the block scope.
pub struct BlockScope<'a> {
    parent: &'a mut dyn Scope,

    /// The referenced names defined in the stack frame but only accessible by children
    names: Vec<(String, BlockNameRef)>
}

#[derive(Copy, Clone)]
enum BlockNameRef {
    Local(StackFrameLocalRef),
    Comptime((StackFrameLocalRef, Option<ComptimeExportRef>))
}

impl <'a> BlockScope<'a> {
    pub fn new(parent: &'a mut dyn Scope) -> Self {
        Self {
            parent,
            names: Vec::new()
        }
    }

    pub fn define_local(&mut self, name: String) -> StackFrameLocalRef {
        let stack_ref = self.parent.define_stack_frame_local();

        self.names.push((name, BlockNameRef::Local(stack_ref)));

        stack_ref
    }

    pub fn define_comptime_main_local(&mut self, name: String) -> StackFrameLocalRef {
        let comptime_main_stack_ref = self.parent.define_comptime_main_stack_frame_local();

        self.names.push((name, BlockNameRef::Comptime((comptime_main_stack_ref, None))));

        comptime_main_stack_ref
    }

    // TODO: Logic:
    //   - When accessing @val from runtime code -> create a slot and provide that slot as result
    pub fn access_local(&mut self, name: &str) -> Result<NameRef, NameAccessError> {
        // By default, code is runtime, so we need to access comptime vals through exports.
        // However, If we pass through a ComptimePortal, then this will get changed to `false`.
        self.access_name(name, true)
    }
}

impl <'a> Scope for BlockScope<'a> {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.parent.define_comptime_main_stack_frame_local()
    }

    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.parent.define_stack_frame_local()
    }

    fn define_comptime_export(&mut self) -> ComptimeExportRef {
        self.parent.define_comptime_export()
    }

    // TODO: Logic:
    //   - When accessing val from runtime code -> return it
    //   - When accessing val from comptime code -> error
    fn access_name(&mut self, name: &str, export_comptime: bool) -> Result<NameRef, NameAccessError> {
        let mut local = None;
        for (i, (local_name, stack_ref)) in self.names.iter().enumerate() {
            if local_name == name {
                local = Some((i, *stack_ref));
                break;
            }
        }

        match local {
            None => self.parent.access_name(name, export_comptime),
            Some((i, BlockNameRef::Local(local_ref))) => Ok(NameRef::Local(local_ref)),
            Some((i, BlockNameRef::Comptime((local_ref, export_ref)))) => {
                if export_comptime {
                    if let Some(export_ref) = export_ref {
                        Ok(NameRef::ComptimeExport(export_ref))
                    } else {
                        let export_ref = self.parent.define_comptime_export();

                        let new_value = BlockNameRef::Comptime((local_ref, Some(export_ref)));
                        let _ = std::mem::replace(&mut self.names[i], (String::from(name), new_value));

                        Ok(NameRef::ComptimeExport(export_ref))

                        // todo!("Define export slot, update the ref to specify it")
                    }
                } else {
                    Ok(NameRef::ComptimeLocal(local_ref))
                }
            },
        }
    }
}




/// This indicates a switch to compile-time code. It acts as a portal to the ComptimeMainStackFrame
/// above it. If a new variable is defined by any block inside, it will not affect the current
/// StackFrame but ComptimeMainStackFrame.
///
/// For example:
///
///     // Root -> ComptimeMainStackFrame [ a ] -> StackFrame [ fn, c ] -> BlockScope
///     // vals: comptime [ a ], local [ fn, c ]
///
///     val c = ...
///
///     @val a = (
///       // Root -> ComptimeMainStackFrame -> StackFrame -> BlockScope -> ComptimePortal -> BlockScope
///       // vals: comptime [ ], local [ b ]
///
///       // This variable is defined in the comptime stack frame instead of the main runtime one
///       val b = 42
///       b
///
///       // If we try to access c here, it should be an error - how?
///     )
///
///     val fn = @{ // this is a ComptimePortal -> StackFrame -> BlockScope
///       // Root -> ComptimeMainStackFrame -> StackFrame -> BlockScope -> ComptimePortal -> StackFrame [ b ] -> BlockScope
///       // vals: comptime [ ], local [ b ]
///
///       // Can access `a`, but needs to capture it
///       a
///
///       // This variable is defined in fn's stack frame, which inherits from the comptime one
///       val b = 42
///
///       (
///         // This is a child BlockScope of fn, any vars defined here still get defined in
///         // fn's stack frame
///         42
///
///         // Can access b directly
///       )
///
///       @(
///         // This is ComptimePortal -> BlockScope again. Any variables defined here will be
///         // defined in the comptime main stack frame, instead of fn's stack frame.
///         42
///
///         // Cannot access b
///       )
///
///       {
///         b + 41
///
///         // Can access b, needs to capture it
///       }
///     }
///
pub struct ComptimePortal<'a> {
    parent: &'a mut BlockScope<'a>
}

impl <'a> ComptimePortal<'a> {
    pub fn new(parent: &'a mut BlockScope<'a>) -> Self {
        ComptimePortal { parent }
    }
}

impl <'a> Scope for ComptimePortal<'a> {
    fn define_comptime_main_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.parent.define_comptime_main_stack_frame_local()
    }

    fn define_stack_frame_local(&mut self) -> StackFrameLocalRef {
        self.parent.define_comptime_main_stack_frame_local()
    }

    fn define_comptime_export(&mut self) -> ComptimeExportRef {
        self.parent.define_comptime_export()
    }

    // TODO: Logic
    //   - When accessing a var above - it needs to be comptime local
    fn access_name(&mut self, name: &str, export_comptime: bool) -> Result<NameRef, NameAccessError> {
        // match self.parent.access_comptime_name(name)? {
        //     ComptimeNameRef::Global(global_ref) => Ok(NameRef::Global(global_ref)),
        //     ComptimeNameRef::Local(local_ref) => Ok(NameRef::Local(local_ref))
        // }

        let parent_ref = self.parent.access_name(name, false)?;
        match parent_ref {
            NameRef::Global(global_ref) => Ok(NameRef::Global(global_ref)),
            NameRef::ComptimeExport(_) => todo!("This shouldn't happen"),
            NameRef::ComptimeLocal(local_ref) => Ok(NameRef::Local(local_ref)),
            NameRef::Local(_) => Err(NameAccessError::CannotReferenceRuntimeNameFromComptime)
        }
    }
}









struct StackFrameLocal {
    // TODO: Include an optional name, for debugging purposes
}

#[derive(Debug, PartialEq)]
pub struct Capture {
    /// The local to capture from the parent stack frame
    from: StackFrameLocalRef,

    /// The local of the child stack frame to put the captured value in
    to: StackFrameLocalRef
}

struct ComptimeExportSlot {}





struct Global {
    name: String
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GlobalRef { i: usize }

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ComptimeExportRef { i: usize }

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StackFrameLocalRef { i: usize }