use crate::frontend::{Location, Pattern};

#[derive(Debug)]
pub struct AST {
    pub value: ASTValue,
    pub location: Location
}

#[derive(Debug)]
pub enum ASTValue {
    Literal(ASTLiteral),
    Block(Box<[AST]>),

    Function {
        params: Box<[ASTParam]>,
        body: Box<AST>,
        return_type: Option<Box<AST>>,
        compile_time: bool,
    },
    Call {
        target: Box<AST>,
        name: Box<str>,
        args: Box<[AST]>,
        maybe_var_call: bool
    },

    Let {
        name: Box<str>,
        value: Box<AST>,
        recursive: bool
    },
    NameRef(Box<str>),

    FnType {
        params: Box<[ASTTypeParam]>,
        return_type: Box<AST>
    },

    TypeAssert {
        value: Box<AST>,
        typ: Box<AST>
    }
}

#[derive(Debug)]
pub enum ASTLiteral {
    Int(i64),
    Bool(bool),
    Float(f64),
    String(Box<str>),
}

#[derive(Debug)]
pub struct ASTParam {
    pub name: Box<str>,
    pub typ: Option<Pattern>,
    pub location: Location
}

#[derive(Debug)]
pub struct ASTTypeParam {
    pub name: Box<str>,
    pub typ: Option<Pattern>,
    pub location: Location
}