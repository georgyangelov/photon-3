use crate::mir;
use crate::mir::lexical_scope::*;
use crate::ast;
use std::borrow::Borrow;
use crate::old_lir::Globals;

#[derive(Debug)]
pub enum CompileError {}

pub struct Compiler {
    pub const_strings: Vec<Box<str>>,

    pub compile_time_main: Vec<mir::MIR>,

    pub functions: Vec<mir::Function>
}

impl Compiler {
    pub fn compile_module(ast: ast::AST, globals: &Globals) -> Result<mir::Module, CompileError> {
        let module_location = ast.location.clone();

        // The module is an implicit function, it's executed like one
        let module_fn = ast::Function {
            params: Vec::new(),
            body: Box::new(ast),

            // TODO: Signal that it doesn't have a return type, not that we don't know it yet
            return_type: None
        };

        // TODO: Populate both of these with the default types like `Int`, `Bool`, `Float`, etc.
        let mut scope = ScopeStack::new(
            RootScope::new(globals.globals.iter().map(|global| global.name.clone()).collect()),
            ComptimeMainStackFrame::new()
        );

        let mut builder = Compiler {
            const_strings: Vec::new(),
            compile_time_main: Vec::new(),
            functions: Vec::new()
        };

        let runtime_main = builder.compile_function(&mut scope, module_fn)?;

        let (root_scope, comptime_main_frame) = scope.consume_root();

        let comptime_main = mir::Function {
            param_types: vec![],
            return_type: None,

            body: mir::MIR {
                node: mir::Node::Block(builder.compile_time_main),
                location: module_location
            },
            param_count: 0,
            local_count: comptime_main_frame.locals.len(),
            captures: vec![]
        };

        Ok(mir::Module {
            functions: builder.functions,

            comptime_export_count: root_scope.comptime_exports.len(),
            comptime_main,

            runtime_main
        })
    }

    fn compile_function(
        &mut self,
        scope: &mut ScopeStack,
        ast: ast::Function
    ) -> Result<mir::Function, CompileError> {
        let param_count = ast.params.len();
        let mut param_names = Vec::with_capacity(param_count);
        let mut param_types = Vec::with_capacity(param_count);
        for param in ast.params {
            param_names.push(String::from(param.name.clone()));
            param_types.push(match param.typ {
                None => None,
                Some(ast::Pattern { value: ast::PatternValue::SpecificValue(ast), .. }) => {
                    let export_ref = self.compile_comptime_expr(scope, ast)?;

                    Some(export_ref)
                }
                _ => todo!("Support patterns in arguments")
            })
        }

        let return_type = match ast.return_type {
            None => None,
            Some(ast) => Some(self.compile_comptime_expr(scope, *ast)?)
        };

        scope.push_stack_frame(param_names);
        scope.push_block();

        let body = self.compile_ast(scope, *ast.body)?;

        scope.pop_block();
        let stack_frame = scope.pop_stack_frame();

        Ok(mir::Function {
            param_count,
            param_types,
            return_type,
            local_count: stack_frame.locals.len(),
            captures: stack_frame.captures,
            body
        })
    }

    fn compile_ast(
        &mut self,
        scope: &mut ScopeStack,
        ast: ast::AST
    ) -> Result<mir::MIR, CompileError> {
        let location = ast.location.clone();
        let node = match ast.value {
            ast::Value::Literal(ast::Literal::Bool(value)) => mir::Node::LiteralBool(value),
            ast::Value::Literal(ast::Literal::Int(value)) => mir::Node::LiteralI64(value),
            ast::Value::Literal(ast::Literal::Float(value)) => mir::Node::LiteralF64(value),

            ast::Value::Literal(ast::Literal::String(value)) => {
                let index = self.const_strings.len();
                self.const_strings.push(value);

                mir::Node::ConstStringRef(index)
            },

            ast::Value::Block(asts) => {
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

            ast::Value::Let { name, value, recursive, comptime } => {
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

                    let set_local_mir = mir::MIR {
                        node: mir::Node::LocalSet(comptime_local_ref, Box::new(mir)),
                        location: location.clone()
                    };

                    self.compile_time_main.push(set_local_mir);

                    mir::Node::Nop
                } else {
                    // val <name> = <value>

                    scope.push_block();

                    let mir = self.compile_ast(scope, *value)?;

                    scope.pop_block();

                    let local_ref = scope.define_local(String::from(name));

                    mir::Node::LocalSet(local_ref, Box::new(mir))
                }
            },

            ast::Value::NameRef(name) => {
                return self.access_name_mir(scope, name.borrow(), location)
                    .map_err(|error| todo!("Compile error - name not found or something else: {:?}, {}", error, name));
            },

            ast::Value::Function(func) => {
                let func_mir = self.compile_function(scope, func)?;
                let func_ref = mir::FunctionRef { i: self.functions.len() };
                let captures = func_mir.captures.clone();

                self.functions.push(func_mir);

                let mut to_capture = Vec::with_capacity(captures.len());
                for capture in captures {
                    to_capture.push(capture.from);
                }

                mir::Node::CreateClosure(func_ref, to_capture)
            },

            ast::Value::Call { name, target, args } => {
                let (target_mir, name) = match target {
                    None => {
                        // fn() is either self.fn() or fn.call(), depends on if there is a name `fn`
                        // in the locals
                        match self.access_name_mir(scope, name.borrow(), location.clone()) {
                            Ok(target) => (target, "call".into()),
                            Err(NameAccessError::NameNotFound) => {
                                match self.access_name_mir(scope, "self", location.clone()) {
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

            ast::Value::If { condition, on_true, on_false } => {
                let condition_mir = self.compile_ast(scope, *condition)?;

                scope.push_block();
                let on_true_mir = self.compile_ast(scope, *on_true)?;
                scope.pop_block();

                scope.push_block();
                let on_false_mir = match on_false {
                    None => None,
                    Some(on_false) => Some(Box::new(self.compile_ast(scope, *on_false)?))
                };
                scope.pop_block();

                mir::Node::If(Box::new(condition_mir), Box::new(on_true_mir), on_false_mir)
            },

            ast::Value::FnType { .. } => todo!("Support fn type definitions"),
            ast::Value::TypeAssert { .. } => todo!("Support type asserts"),

            ast::Value::CompileTimeExpr(ast) => {
                let export_ref = self.compile_comptime_expr(scope, *ast)?;

                mir::Node::CompileTimeGet(export_ref)
            }
        };

        Ok(mir::MIR {
            node,
            location
        })
    }

    fn compile_comptime_expr(
        &mut self,
        scope: &mut ScopeStack,
        ast: ast::AST
    ) -> Result<mir::ComptimeExportRef, CompileError> {
        let location = ast.location.clone();
        let export_ref = scope.define_comptime_export();

        scope.push_comptime_portal();
        scope.push_block();

        let mir = self.compile_ast(scope, ast)?;

        scope.pop_block();
        scope.pop_comptime_portal();

        let comptime_code = mir::Node::CompileTimeSet(export_ref, Box::new(mir));

        self.compile_time_main.push(mir::MIR {
            node: comptime_code,
            location
        });

        Ok(export_ref)
    }

    fn access_name_mir(&mut self, scope: &mut ScopeStack, name: &str, location: ast::Location) -> Result<mir::MIR, NameAccessError> {
        let node = match scope.access_local(name) {
            Err(error) => return Err(error),
            Ok(AccessNameRef::ComptimeExport(export_ref, first_access)) => {
                if let Some(comptime_local_ref) = first_access {
                    let get_comptime_local = mir::MIR {
                        node: mir::Node::LocalGet(comptime_local_ref),
                        location: location.clone()
                    };
                    let set_comptime_slot = mir::Node::CompileTimeSet(export_ref, Box::new(get_comptime_local));

                    self.compile_time_main.push(mir::MIR {
                        node: set_comptime_slot,
                        location: location.clone()
                    });
                }

                mir::Node::CompileTimeGet(export_ref)
            },
            Ok(AccessNameRef::Global(global_ref)) => mir::Node::GlobalRef(global_ref),
            Ok(AccessNameRef::Capture(capture_ref)) => mir::Node::CaptureRef(capture_ref),
            Ok(AccessNameRef::Param(param_ref)) => mir::Node::ParamRef(param_ref),
            Ok(AccessNameRef::Local(local_ref)) => mir::Node::LocalGet(local_ref),
        };

        Ok(mir::MIR { node, location })
    }
}
