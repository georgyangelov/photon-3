// use crate::compiler::lexical_scope::{BlockScope, FnScope, LexicalScope, LocalSlotRef, RootScope, SlotRef};
// use crate::compiler::{lir, mir};
// use crate::frontend::{AST, ASTFunction, ASTLiteral, ASTValue};
//
// pub enum CompileError {}
//
// struct FunctionTemplate {
//     body: AST
// }
//
// pub struct ModuleCompiler<'a> {
//     pub const_strings: Vec<Box<str>>,
//
//     // pub compile_time_slots: Vec<Any>,
//     pub compile_time_functions: Vec<lir::Function>,
//     pub compile_time_scope: BlockScope<'a>,
//     pub compile_time_main: Vec<mir::MIR>,
//
//     pub run_time_functions: Vec<mir::Function>
// }
//
// pub struct Module {
//     pub compile_time_functions: Vec<lir::Function>,
//
//     // pub compile_time_main: lir::Function,
//     //
//     pub run_time_functions: Vec<mir::Function>,
//     // pub run_time_main: mir::Function
// }
//
// impl <'a> ModuleCompiler<'a> {
//     fn compile_module(ast: AST) -> Result<Module, CompileError> {
//         // The module is an implicit function, it's executed like one
//         let module_fn = ASTFunction {
//             params: Vec::new(),
//             body: Box::new(ast),
//
//             // TODO: Signal that it doesn't have a return type, not that we don't know it yet
//             return_type: None
//         };
//
//         // TODO: Populate both of these with the default types like `Int`, `Bool`, `Float`, etc.
//         let mut scope = RootScope::new();
//         let mut compile_time_root_scope = RootScope::new();
//         let mut compile_time_fn_scope = FnScope::new(&mut compile_time_root_scope, vec![]);
//
//         let mut builder = ModuleCompiler {
//             const_strings: Vec::new(),
//             compile_time_functions: Vec::new(),
//             compile_time_scope: BlockScope::new(&mut compile_time_fn_scope),
//             compile_time_main: Vec::new(),
//             run_time_functions: Vec::new()
//         };
//
//
//
//         let compiled = builder.compile_function(&mut scope, module_fn)?;
//
//         Ok(Module {
//             compile_time_functions: builder.compile_time_functions,
//             run_time_functions: builder.run_time_functions
//         })
//     }
//
//     fn compile_function(
//         &mut self,
//         parent_scope: &mut dyn LexicalScope,
//         ast: ASTFunction
//     ) -> Result<mir::Function, CompileError> {
//         let param_names = ast.params.iter().map(|p| String::from(*p.name)).collect();
//         let mut fn_scope = FnScope::new(parent_scope, param_names);
//         let mut block_scope = BlockScope::new(&mut fn_scope);
//
//         let body = self.compile_ast(&mut block_scope, *ast.body)?;
//
//         Ok(mir::Function {
//             body:
//         })
//     }
//
//     fn compile_ast(
//         &mut self,
//         scope: &mut BlockScope,
//         ast: AST
//     ) -> Result<CompileASTResult, CompileError> {
//         let location = ast.location;
//         let node = match ast.value {
//             ASTValue::Literal(ASTLiteral::Bool(value)) => mir::Node::LiteralI32(if value { 1 } else { 0 }),
//             ASTValue::Literal(ASTLiteral::Int(value)) => mir::Node::LiteralI64(value),
//             ASTValue::Literal(ASTLiteral::Float(value)) => mir::Node::LiteralF64(value),
//
//             ASTValue::Literal(ASTLiteral::String(value)) => {
//                 let offset = self.const_strings.len();
//                 self.const_strings.push(value);
//
//                 mir::Node::ConstStringRef(offset)
//             },
//
//             ASTValue::Block(asts) => {
//                 let mut inner_scope = BlockScope::new(scope);
//
//                 let mut mirs = Vec::with_capacity(asts.len());
//                 let len = asts.len();
//                 for (i, ast) in asts.into_iter().enumerate() {
//                     let is_last = i == len - 1;
//                     let mir = self.compile_ast(&mut inner_scope, ast)?;
//
//                     // TODO: Flatten nested blocks?
//                     // TODO: What if we have a block where the last value is a compile-time assignment
//
//                     match mir {
//                         CompileASTResult::CompileTimeValue(local_ref) => {
//                             if is_last {
//                                 mirs.push(todo!("Copy value into a compile-time slot"))
//                             }
//                         }
//
//                         CompileASTResult::RunTimeValue(mir) => mirs.push(mir)
//                     }
//                 }
//
//                 // if mirs.len() == 0 {
//                 //     // This is a weird case - we seem to have a block consisting only of
//                 //     // compile-time assignments, so there is no actual code to be executed at runtime.
//                 //     mir::Node::Nop
//                 // } else {
//                 mir::Node::Block(mirs)
//                 // }
//             },
//
//             // TODO: Support rec val
//             ASTValue::Let { name, value, recursive } => {
//                 let value_mir = self.compile_ast(scope, *value)?;
//
//                 match value_mir {
//                     CompileASTResult::CompileTimeValue(_) => {
//                         // The val is a compile-time one
//                         // val a = @expr
//                         // So nothing to do here for now, when someone uses this val in a runtime
//                         // context - it will be loaded then.
//                         mir::Node::Block(vec![])
//                     }
//
//                     CompileASTResult::RunTimeValue(mir) => {
//                         let slot_ref = scope.define_name(Some(name.into_string()));
//
//                         mir::Node::LocalSet(slot_ref, Box::new(mir))
//                     }
//                 }
//             },
//
//             ASTValue::NameRef(name) => {
//                 let name_ref = scope.access_name(name.borrow());
//
//                 match name_ref {
//                     None => todo!("Compile error - name not found"),
//                     Some(SlotRef::CompileTime(comptime_slot_ref)) => mir::Node::CompileTimeRef(comptime_slot_ref),
//                     Some(SlotRef::CompileTimeLocal(local_ref)) => {
//                         // Let the scope know that we've already exported this - further loads
//                         // will fall in the case above
//                         let comptime_slot_ref = scope.define_compile_time_export_name(name.into_string());
//
//                         let load_local = mir::MIR {
//                             node: mir::Node::LocalGet(local_ref),
//                             location: location.clone()
//                         };
//
//                         let copy_local_to_slot = mir::Node::CompileTimeSet(
//                             comptime_slot_ref,
//                             Box::new(load_local)
//                         );
//
//                         self.compile_time_main.push(mir::MIR {
//                             node: copy_local_to_slot,
//                             location: location.clone()
//                         });
//
//                         mir::Node::CompileTimeRef(comptime_slot_ref)
//                     },
//                     Some(SlotRef::Global(global_slot_ref)) => mir::Node::GlobalRef(global_slot_ref),
//                     Some(SlotRef::Local(local_ref)) => mir::Node::LocalGet(local_ref),
//                 }
//             },
//
//             ASTValue::Function(_) => {}
//             ASTValue::Call { .. } => {}
//
//             ASTValue::FnType { .. } => {}
//             ASTValue::TypeAssert { .. } => {}
//
//             ASTValue::CompileTimeExpr(ast) => {
//                 let mir = self.compile_ast()
//             }
//         };
//
//         Ok(Some(mir::MIR {
//             node,
//             location
//         }))
//     }
// }
//
// enum CompileASTResult {
//     CompileTimeValue(LocalSlotRef),
//     RunTimeValue(mir::MIR)
// }
//
//
//
//
//
//
//
