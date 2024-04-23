use crate::frontend::{Location, Pattern};

pub struct AST {
    pub value: ASTValue,
    pub location: Location
}

pub enum ASTValue {
    Literal(ASTLiteral),
    Block(Box<[AST]>),

    Function {
        params: Box<[ASTParam]>,
        body: AST,
        return_type: Option<AST>,
        compile_time: bool,
    },
    Call {
        target: AST,
        name: Box<str>,
        args: Box<[AST]>,
        maybe_var_call: bool
    },

    Let {
        name: Box<str>,
        value: AST,
        recursive: bool
    },
    NameRef(Box<str>),

    FnType {
        params: Box<[ASTTypeParam]>,
        return_type: AST
    },

    TypeAssert {
        value: AST,
        typ: AST
    }
}

pub enum ASTLiteral {
    Int(i64),
    Bool(bool),
    Float(f64),
    String(Box<str>),
}

pub struct ASTParam {
    pub name: Box<str>,
    pub typ: Option<Pattern>,
    pub location: Location
}

pub struct ASTTypeParam {
    pub name: Box<str>,
    pub typ: Option<Pattern>,
    pub location: Location
}