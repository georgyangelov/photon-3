use crate::frontend::{Location, Pattern};

#[derive(Debug)]
pub struct AST {
    pub value: ASTValue,
    pub location: Location
}

#[derive(Debug)]
pub enum ASTValue {
    Literal(ASTLiteral),
    Block(Vec<AST>),

    Function {
        params: Vec<ASTParam>,
        body: Box<AST>,
        return_type: Option<Box<AST>>
    },
    Call {
        // may_be_var_call == this being None
        target: Option<Box<AST>>,
        name: Box<str>,
        args: Vec<AST>
    },

    Let {
        name: Box<str>,
        value: Box<AST>,
        recursive: bool
    },
    NameRef(Box<str>),

    FnType {
        params: Vec<ASTTypeParam>,
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
    pub typ: AST,
    pub location: Location
}