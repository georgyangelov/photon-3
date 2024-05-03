use crate::compiler::{mir};
use crate::compiler::lexical_scope::{AccessNameRef, ComptimeMainStackFrame, NameAccessError, RootScope, ScopeStack};
use crate::frontend::{AST, ASTFunction, ASTLiteral, ASTValue, Location};
use std::borrow::Borrow;
use crate::compiler::mir::{FrameLayout, FunctionRef, MIR};

#[derive(Debug)]
pub enum CompileError {}

struct FunctionTemplate {
    body: AST
}

pub struct ModuleCompiler {
    pub const_strings: Vec<Box<str>>,

    // pub compile_time_slots: Vec<Any>,
    // pub compile_time_functions: Vec<lir::Function>,
    // pub compile_time_scope: BlockScope<'a>,
    pub compile_time_main: Vec<MIR>,

    pub runtime_functions: Vec<mir::Function>
}

pub struct Module {
    // pub compile_time_functions: Vec<lir::Function>,

    // pub compile_time_main: lir::Function,

    pub runtime_functions: Vec<mir::Function>,
    pub runtime_main: mir::Function
}

impl ModuleCompiler {
    pub fn compile_module(ast: AST) -> Result<Module, CompileError> {
        // The module is an implicit function, it's executed like one
        let module_fn = ASTFunction {
            params: Vec::new(),
            body: Box::new(ast),

            // TODO: Signal that it doesn't have a return type, not that we don't know it yet
            return_type: None
        };

        // TODO: Populate both of these with the default types like `Int`, `Bool`, `Float`, etc.
        let mut scope = ScopeStack::new(
            RootScope::new(),
            ComptimeMainStackFrame::new()
        );

        let mut builder = ModuleCompiler {
            const_strings: Vec::new(),
            compile_time_main: Vec::new(),
            runtime_functions: Vec::new()
        };

        let compiled = builder.compile_function(&mut scope, module_fn)?;

        Ok(Module {
            // compile_time_functions: builder.compile_time_functions,
            // run_time_functions: builder.run_time_functions
            runtime_functions: builder.runtime_functions,
            runtime_main: compiled
        })
    }

    fn compile_function(
        &mut self,
        scope: &mut ScopeStack,
        ast: ASTFunction
    ) -> Result<mir::Function, CompileError> {
        scope.push_stack_frame();
        scope.push_block();

        for param in ast.params {
            scope.define_local(String::from(param.name));
        }

        let body = self.compile_ast(scope, *ast.body)?;

        scope.pop_block();
        let stack_frame = scope.pop_stack_frame();

        Ok(mir::Function {
            frame_layout: FrameLayout {
                size: stack_frame.locals.len()
            },
            captures: stack_frame.captures,
            body
        })
    }

    fn compile_ast(
        &mut self,
        scope: &mut ScopeStack,
        ast: AST
    ) -> Result<mir::MIR, CompileError> {
        let location = ast.location.clone();
        let node = match ast.value {
            ASTValue::Literal(ASTLiteral::Bool(value)) => mir::Node::LiteralI8(if value { 1 } else { 0 }),
            ASTValue::Literal(ASTLiteral::Int(value)) => mir::Node::LiteralI64(value),
            ASTValue::Literal(ASTLiteral::Float(value)) => mir::Node::LiteralF64(value),

            ASTValue::Literal(ASTLiteral::String(value)) => {
                let index = self.const_strings.len();
                self.const_strings.push(value);

                mir::Node::ConstStringRef(index)
            },

            ASTValue::Block(asts) => {
                scope.push_block();

                let mut mirs = Vec::with_capacity(asts.len());
                for ast in asts {
                    let mir = self.compile_ast(scope, ast)?;

                    // TODO: Flatten nested blocks?
                    // TODO: What if we have a block where the last value is a compile-time assignment

                    if !matches!(mir.node, mir::Node::Nop) {
                        mirs.push(mir)
                    }
                }

                scope.pop_block();

                if mirs.len() == 0 {
                    // This is a weird case - we seem to have a block consisting only of
                    // compile-time assignments, so there is no actual code to be executed at runtime.
                    mir::Node::Nop
                } else {
                    mir::Node::Block(mirs)
                }
            },

            ASTValue::Let { name, value, recursive, comptime } => {
                if recursive {
                    todo!("Support rec vals - should be enough to call define before compile_ast(value)")
                }

                // TODO: Refactor to unify the two cases
                if comptime {
                    // @val <name> = <value>

                    scope.push_comptime_portal();
                    scope.push_block();

                    let mir = self.compile_ast(scope, *value)?;

                    scope.pop_block();
                    scope.pop_comptime_portal();

                    let comptime_local_ref = scope.define_comptime_main_local(String::from(name));

                    mir::Node::LocalSet(comptime_local_ref, Box::new(mir))
                } else {
                    // val <name> = <value>

                    scope.push_block();

                    let mir = self.compile_ast(scope, *value)?;

                    scope.pop_block();

                    let local_ref = scope.define_local(String::from(name));

                    mir::Node::LocalSet(local_ref, Box::new(mir))
                }
            },

            ASTValue::NameRef(name) => {
                return Self::access_name_mir(scope, name.borrow(), location)
                    .map_err(|error| todo!("Compile error - name not found or something else"));
            },

            ASTValue::Function(func) => {
                let func_mir = self.compile_function(scope, func)?;
                let func_ref = FunctionRef { i: self.runtime_functions.len() };
                let captures = func_mir.captures.clone();

                self.runtime_functions.push(func_mir);

                let mut locals_to_capture = Vec::with_capacity(captures.len());
                for capture in captures {
                    locals_to_capture.push(capture.from);
                }

                mir::Node::CreateClosure(func_ref, locals_to_capture)
            },

            ASTValue::Call { name, target, args } => {
                let (target_mir, name) = match target {
                    None => {
                        // fn() is either self.fn() or fn.call(), depends on if there is a name `fn`
                        // in the locals
                        match Self::access_name_mir(scope, name.borrow(), location.clone()) {
                            Ok(target) => (target, "call".into()),
                            Err(NameAccessError::NameNotFound) => {
                                match Self::access_name_mir(scope, "self", location.clone()) {
                                    Ok(target_self) => (target_self, name),
                                    Err(_) => {
                                        println!("{:?}", name);
                                        todo!("Compile error - could not find function")
                                    }
                                }
                            },
                            Err(_) => todo!("Compile error")
                        }
                    },
                    Some(target) => (self.compile_ast(scope, *target)?, name)
                };

                let mut args_mir = Vec::new();
                for arg in args {
                    let mir = self.compile_ast(scope, arg)?;

                    args_mir.push(mir);
                }

                mir::Node::Call(name, Box::new(target_mir), args_mir)
            },

            ASTValue::FnType { .. } => todo!("Support fn type definitions"),
            ASTValue::TypeAssert { .. } => todo!("Support type asserts"),

            ASTValue::CompileTimeExpr(ast) => {
                let export_ref = scope.define_comptime_export();

                scope.push_comptime_portal();
                scope.push_block();

                let mir = self.compile_ast(scope, *ast)?;

                scope.pop_block();
                scope.pop_comptime_portal();

                let comptime_code = mir::Node::CompileTimeSet(export_ref, Box::new(mir));

                self.compile_time_main.push(mir::MIR {
                    node: comptime_code,
                    location: location.clone()
                });

                mir::Node::CompileTimeRef(export_ref)
            }
        };

        Ok(mir::MIR {
            node,
            location
        })
    }

    fn access_name_mir(scope: &mut ScopeStack, name: &str, location: Location) -> Result<mir::MIR, NameAccessError> {
        let node = match scope.access_local(name) {
            Err(error) => return Err(error),
            Ok(AccessNameRef::ComptimeExport(export_ref)) => mir::Node::CompileTimeRef(export_ref),
            Ok(AccessNameRef::Global(global_ref)) => mir::Node::GlobalRef(global_ref),
            Ok(AccessNameRef::Local(local_ref)) => mir::Node::LocalGet(local_ref),
        };

        Ok(mir::MIR { node, location })
    }
}
