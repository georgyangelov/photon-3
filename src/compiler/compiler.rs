use std::collections::HashMap;
use crate::compiler::mir::{FunctionRef, MIR, MIRType};
use crate::frontend::AST;

pub struct Compiler {
    functions: HashMap<String, Function>
}

pub enum CompileError {}

pub struct Function {
    fn_ref: FunctionRef,
    name: String,
    return_type: MIRType
}

impl Compiler {
    fn compile(&mut self, ast: AST) -> Result<MIR, CompileError> {

    }
}