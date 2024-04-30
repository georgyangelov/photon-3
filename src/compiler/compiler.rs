// use std::collections::HashMap;
use crate::compiler::lexical_scope::{Capture, LexicalScope};
use crate::compiler::mir::{FunctionRef, MIR, MIRType};
use crate::frontend::{AST, ASTFunction};

pub struct Compiler {
    // functions: HashMap<String, Function>
}

pub enum CompileError {}

pub struct Function {
    // fn_ref: FunctionRef,
    // name: String,
    captures: Vec<Capture>,
    return_type: MIRType
}

impl Compiler {
    fn compile_function(&mut self, parent_scope: &mut LexicalScope, ast_fn: ASTFunction) -> Result<Function, CompileError> {
        ast_fn.params

        Ok(Function {

        })
    }

    fn compile(&mut self, scope: LexicalScope, ast: AST) -> Result<MIR, CompileError> {

    }
}