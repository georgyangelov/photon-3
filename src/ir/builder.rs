use crate::{ast, ir};
use crate::ir::{Globals, Value};
use crate::ir::lexical_scope::{NameAccessError, NameRef, RootScope, ScopeStack};
use crate::vec_map::VecMap;

pub struct Builder {
    pub functions: Vec<ir::TFunction>
}

impl Builder {
    // TODO: Actual error handling instead of panics
    pub fn build_module(ast: ast::AST, globals: &Globals) -> ir::PreComptimeModule {
        let main_fn_ast = ast::Function {
            params: Vec::new(),
            body: Box::new(ast),

            // TODO: Signal that it doesn't have a return type, not that we don't know it yet
            return_type: None
        };

        let mut scope = ScopeStack::new(
            RootScope::new(globals.globals.clone())
        );

        let mut builder = Builder {
            functions: Vec::new()
        };

        let main = builder.build_function(&mut scope, main_fn_ast);

        ir::PreComptimeModule {
            functions: builder.functions,
            main
        }
    }

    fn build_function(&mut self, scope: &mut ScopeStack, ast: ast::Function) -> ir::TFunction {
        let param_count = ast.params.len();

        let mut scope_params = Vec::with_capacity(param_count);
        for param in &ast.params {
            scope_params.push(ir::lexical_scope::Param {
                name: String::from(param.name.as_ref()),
                comptime: param.comptime
            });
        }

        scope.push_stack_frame(scope_params);

        let mut params = VecMap::with_capacity(param_count);
        for (i, param) in ast.params.into_iter().enumerate() {
            let param_ref = ir::ParamRef { i, comptime: param.comptime };

            // TODO: Do we need this block to be here?
            scope.push_block();

            let param_type = match param.typ {
                None => None,
                Some(ast::Pattern { value: ast::PatternValue::SpecificValue(ast), .. }) => {
                    let ir = self.compile_implicit_comptime_ast(scope, ast);

                    Some(Box::new(ir))
                }
                _ => todo!("Support patterns in arguments")
            };

            scope.pop_block();

            params.insert_push(param_ref, ir::TParam {
                typ: param_type,
                comptime: param.comptime
            });
        }

        let return_type = match ast.return_type {
            None => None,
            Some(ast) => Some(self.compile_implicit_comptime_ast(scope, *ast))
        };

        scope.push_block();

        let body = self.build_ir(scope, *ast.body);

        scope.pop_block();
        let stack_frame = scope.pop_stack_frame();

        let mut captures = VecMap::with_capacity(stack_frame.captures.len());
        for (i, capture) in stack_frame.captures.into_iter().enumerate() {
            // TODO: Do we need for these refs to have their own `comptime`?
            let capture_ref = ir::CaptureRef { i, comptime: capture.comptime };

            captures.insert_push(capture_ref, ir::TCapture {
                from: capture.from,
                comptime: capture.comptime
            });
        }

        let mut locals = VecMap::with_capacity(stack_frame.locals.len());
        for (i, local) in stack_frame.locals.iter().enumerate() {
            // TODO: Maybe the `lexical_scope` should be creating those Refs
            let local_ref = ir::LocalRef { i, comptime: local.comptime };

            locals.insert_push(local_ref, ir::TLocal {
                comptime: local.comptime
            });
        }

        ir::TFunction { captures, params, return_type, locals, body }
    }

    fn build_ir(&mut self, scope: &mut ScopeStack, ast: ast::AST) -> ir::IR {
        let location = ast.location.clone();
        let node = match ast.value {
            ast::Value::Literal(ast::Literal::Bool(value)) => ir::Node::Constant(Value::Bool(value)),
            ast::Value::Literal(ast::Literal::Int(value)) => ir::Node::Constant(Value::Int(value)),
            ast::Value::Literal(ast::Literal::Float(value)) => ir::Node::Constant(Value::Float(value)),
            ast::Value::Literal(ast::Literal::String(_)) => todo!("Support compiling string literals"),

            ast::Value::Block(asts) => {
                scope.push_block();

                let mut irs = Vec::with_capacity(asts.len());
                for ast in asts {
                    let ir = self.build_ir(scope, ast);

                    // TODO: Flatten nested blocks?
                    // TODO: What if we have a block where the last value is a compile-time assignment

                    if !matches!(ir.node, ir::Node::Nop) {
                        irs.push(ir)
                    }
                }

                scope.pop_block();

                if irs.len() == 0 {
                    // This is a weird case - we seem to have a block consisting only of
                    // compile-time assignments, so there is no actual code to be executed at runtime.
                    ir::Node::Nop
                } else {
                    ir::Node::Block(irs)
                }
            }

            ast::Value::Let { name, value, recursive, comptime } => {
                if recursive {
                    todo!("Remove recursive lets")
                }

                if comptime {
                    scope.push_comptime_portal();
                }
                scope.push_block();

                let ir = self.build_ir(scope, *value);

                scope.pop_block();
                if comptime {
                    scope.pop_comptime_portal();
                }

                let local_ref = scope.define_local(String::from(name), comptime);

                ir::Node::LocalSet(local_ref, Box::new(ir))
            }

            ast::Value::NameRef(name) => self.lookup_ir(scope, name.as_ref()).expect("Cannot find name"),

            ast::Value::Function(func) => {
                let func_ir = self.build_function(scope, func);
                let func_ref = ir::FunctionTemplateRef { i: self.functions.len() };
                let captures = func_ir.captures.clone();

                self.functions.push(func_ir);

                let mut to_capture = VecMap::with_capacity(captures.len());
                for (capture_ref, capture) in captures.iter() {
                    to_capture.insert_push(*capture_ref, capture.from);
                }

                ir::Node::DynamicCreateClosure(func_ref, to_capture)
            }

            ast::Value::Call { name, target, args } => {
                let (target_ir, name) = match target {
                    None => {
                        // fn() is either self.fn() or fn.call(), depends on if there is a name `fn`
                        // in the locals
                        match self.lookup_ir(scope, name.as_ref()) {
                            Ok(target) => (ir::IR { node: target, location: location.clone() }, "call".into()),
                            Err(NameAccessError::NameNotFound) => {
                                match self.lookup_ir(scope, "self") {
                                    Ok(target_self) => (ir::IR { node: target_self, location: location.clone() }, name),
                                    Err(_) => {
                                        println!("{:?}", name);
                                        todo!("Compile error - could not find function")
                                    }
                                }
                            },
                            Err(_) => todo!("Compile error")
                        }
                    }
                    Some(target) => (self.build_ir(scope, *target), name)
                };

                let mut args_ir = Vec::new();
                for arg in args {
                    let ir = self.build_ir(scope, arg);

                    args_ir.push(ir);
                }

                ir::Node::DynamicCall(name, Box::new(target_ir), args_ir)
            }

            ast::Value::If { condition, on_true, on_false } => {
                let condition_ir = self.build_ir(scope, *condition);

                scope.push_block();
                let on_true_ir = self.build_ir(scope, *on_true);
                scope.pop_block();

                scope.push_block();
                let on_false_ir = match on_false {
                    None => None,
                    Some(on_false) => Some(Box::new(self.build_ir(scope, *on_false)))
                };
                scope.pop_block();

                ir::Node::If(Box::new(condition_ir), Box::new(on_true_ir), on_false_ir)
            }

            ast::Value::FnType { .. } => todo!("Support fn type definitions"),
            ast::Value::TypeAssert { .. } => todo!("Support type asserts"),

            ast::Value::CompileTimeExpr(ast) => self.compile_comptime_ast(scope, *ast)
        };

        ir::IR { node, location }
    }

    fn compile_implicit_comptime_ast(&mut self, scope: &mut ScopeStack, ast: ast::AST) -> ir::IR {
        let location = ast.location.clone();
        let node = self.compile_comptime_ast(scope, ast);

        ir::IR { node, location }
    }

    fn compile_comptime_ast(&mut self, scope: &mut ScopeStack, ast: ast::AST) -> ir::Node {
        scope.push_comptime_portal();
        let ir = self.build_ir(scope, ast);
        scope.pop_comptime_portal();

        ir::Node::Comptime(Box::new(ir))
    }

    fn lookup_ir(&mut self, scope: &mut ScopeStack, name: &str) -> Result<ir::Node, NameAccessError> {
        Ok(match scope.lookup(name)? {
            NameRef::Global(global_ref) => ir::Node::GlobalRef(global_ref),
            NameRef::Capture(capture_ref) => ir::Node::CaptureRef(capture_ref),
            NameRef::Param(param_ref) => ir::Node::ParamRef(param_ref),
            NameRef::Local(local_ref) => ir::Node::LocalRef(local_ref)
        })
    }
}