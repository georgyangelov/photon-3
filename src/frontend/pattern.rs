use crate::frontend::{AST, Location};

#[derive(Debug)]
pub struct Pattern {
    pub value: PatternValue,
    pub location: Location
}

#[derive(Debug)]
pub enum PatternValue {
    SpecificValue(AST),
    Binding(Box<str>),
    Call {
        // may_be_var_call == this being None
        target: Option<Box<AST>>,
        name: Box<str>,
        args: Vec<Pattern>
    },
    FunctionType {
        params: Vec<PatternParam>,
        return_type: Box<Pattern>
    }
}

#[derive(Debug)]
pub struct PatternParam {
    pub name: Box<str>,
    pub typ: Pattern,
    pub location: Location
}