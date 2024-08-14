use crate::ast::AST;
use crate::{ast, ir};
use crate::ir::{Globals, Type, Value};
use crate::ir::IR;

// pub struct Compiler {
//     pub functions: Vec<ir::Function>,
//     pub main: Vec<IR>
// }
//
// impl Compiler {
//     // TODO: Actual error handling instead of panics
//     pub fn compile_module(ast: AST, globals: &Globals) -> ir::Module {
//         let module_location = ast.location.clone();
//
//         let main_ast_fn = ast::Function {
//             params: Vec::new(),
//             body: Box::new(ast),
//
//             // TODO: Signal that it doesn't have a return type, not that we don't know it yet
//             return_type: None
//         };
//
//
//     }
// }