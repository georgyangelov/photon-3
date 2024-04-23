use crate::frontend::{AST, Location};

#[derive(Debug)]
pub struct Pattern {
    pub value: PatternValue,
    pub location: Location
}

pub enum PatternValue {
    SpecificValue(AST),
    Binding(Box<str>),
    Call {
        target: AST,
        name: Box<str>,
        args: Box<[Pattern]>,
        maybe_var_call: bool
    },
    FunctionType {
        params: Box<[PatternParam]>,
        return_type: Pattern
    }
}

pub struct PatternParam {
    pub name: Box<str>,
    pub typ: Pattern
}