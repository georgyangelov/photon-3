use crate::compiler::lexical_scope::{FnScope, LexicalScope};
use crate::compiler::{lir, mir};
use crate::compiler::lir::Any;
use crate::frontend::{AST, ASTFunction, ASTLiteral, ASTValue};

// pub struct ModuleCompiler {
//     // functions: HashMap<String, Function>
//     function_templates: Vec<FunctionTemplate>
// }

pub enum CompileError {}

struct FunctionTemplate {
    body: AST
}

pub struct ModuleCompiler {
    pub const_strings: Vec<Box<str>>,

    // pub compile_time_slots: Vec<Any>,
    pub compile_time_functions: Vec<lir::Function>,
    pub compile_time_main: Vec<mir::MIR>,

    pub run_time_functions: Vec<mir::Function>
}

pub struct Module {
    pub compile_time_functions: Vec<lir::Function>,

    // pub compile_time_main: lir::Function,
    //
    pub run_time_functions: Vec<mir::Function>,
    // pub run_time_main: mir::Function
}

impl ModuleCompiler {
    fn compile_module(ast: AST) -> Result<Module, CompileError> {
        // The module is an implicit function, it's executed like one
        let module_fn = ASTFunction {
            params: Vec::new(),
            body: Box::new(ast),

            // TODO: Signal that it doesn't have a return type, not that we don't know it yet
            return_type: None
        };

        // TODO: Populate both of these with the default types like `Int`, `Bool`, `Float`, etc.
        let compile_time_scope = LexicalScope::new_root();
        let run_time_scope = LexicalScope::new_root();

        let mut builder = ModuleCompiler {
            const_strings: Vec::new(),
            compile_time_functions: Vec::new(),
            run_time_functions: Vec::new()
        };

        let compiled = builder.compile_function(
            compile_time_scope,
            run_time_scope,
            module_fn
        )?;

        Ok(Module {
            compile_time_functions: builder.compile_time_functions,
            run_time_functions: builder.run_time_functions
        })
    }

    fn compile_function(
        &mut self,
        mut c_scope: FnScope,
        mut r_scope: FnScope,
        ast: ASTFunction
    ) -> Result<mir::Function, CompileError> {
        let c_lex_scope = LexicalScope::Fn(&mut c_scope);
        let r_lex_scope = LexicalScope::Fn(&mut r_scope);

        let body = self.compile_ast(c_lex_scope, r_lex_scope, *ast.body)?;

        Ok(mir::Function {
            body:
        })
    }

    fn compile_ast(
        &mut self,
        scope: &mut LexicalScope,
        ast: AST
    ) -> Result<Option<mir::MIR>, CompileError> {
        let node = match ast.value {
            ASTValue::Literal(ASTLiteral::Bool(value)) => mir::Node::LiteralI32(if value { 1 } else { 0 }),
            ASTValue::Literal(ASTLiteral::Int(value)) => mir::Node::LiteralI64(value),
            ASTValue::Literal(ASTLiteral::Float(value)) => mir::Node::LiteralF64(value),

            ASTValue::Literal(ASTLiteral::String(value)) => {
                let offset = self.const_strings.len();
                self.const_strings.push(value);

                mir::Node::ConstStringRef(offset)
            },

            ASTValue::Block(asts) => {
                let mut mirs = Vec::with_capacity(asts.len());
                for ast in asts {
                    let mut inner_c_scope = c_scope.new_child_block();
                    let mut inner_r_scope = r_scope.new_child_block();

                    let mut inner_c_lex_scope = LexicalScope::Block(&mut inner_c_scope);
                    let mut inner_r_lex_scope = LexicalScope::Block(&mut inner_r_scope);

                    let mir = self.compile_ast(
                        &mut inner_c_lex_scope,
                        &mut inner_r_lex_scope,
                        ast
                    )?;

                    if let Some(mir) = mir {
                        mirs.push(mir);
                    }
                }

                mir::Node::Block(mirs)
            },

            ASTValue::Let { name, value, recursive } => {
                let value_mir = self.compile_ast(
                    c_scope,
                    r_scope,

                )
            },

            ASTValue::NameRef(_) => {}

            ASTValue::Function(_) => {}
            ASTValue::Call { .. } => {}

            ASTValue::FnType { .. } => {}
            ASTValue::TypeAssert { .. } => {}
        };

        Ok(Some(mir::MIR {
            node,
            location: ast.location
        }))
    }
}









