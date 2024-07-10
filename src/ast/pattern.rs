use crate::ast;

#[derive(Debug, Clone)]
pub struct Pattern {
    pub value: PatternValue,
    pub location: ast::Location
}

#[derive(Debug, Clone)]
pub enum PatternValue {
    SpecificValue(ast::AST),
    Binding(Box<str>),
    Call {
        // may_be_var_call == this being None
        target: Option<Box<ast::AST>>,
        name: Box<str>,
        args: Vec<Pattern>
    },
    FunctionType {
        params: Vec<PatternParam>,
        return_type: Box<Pattern>
    }
}

#[derive(Debug, Clone)]
pub struct PatternParam {
    pub name: Box<str>,
    pub typ: Pattern,
    pub location: ast::Location
}