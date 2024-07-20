use crate::ast;

#[derive(Debug, Clone)]
pub struct AST {
    pub value: Value,
    pub location: ast::Location
}

#[derive(Debug, Clone)]
pub enum Value {
    Literal(Literal),
    Block(Vec<AST>),

    Function(Function),
    Call {
        // may_be_var_call == this being None
        target: Option<Box<AST>>,
        name: Box<str>,
        args: Vec<AST>
    },

    Let {
        name: Box<str>,
        value: Box<AST>,
        recursive: bool,
        comptime: bool
    },
    NameRef(Box<str>),

    If {
        condition: Box<AST>,
        on_true: Box<AST>,
        on_false: Option<Box<AST>>
    },

    FnType {
        params: Vec<TypeParam>,
        return_type: Box<AST>
    },

    TypeAssert {
        value: Box<AST>,
        typ: Box<AST>
    },

    CompileTimeExpr(Box<AST>)
}

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<Param>,
    pub body: Box<AST>,
    pub return_type: Option<Box<AST>>
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Bool(bool),
    Float(f64),
    String(Box<str>),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Box<str>,
    pub comptime: bool,
    pub typ: Option<ast::Pattern>,
    pub location: ast::Location
}

#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: Box<str>,
    pub typ: AST,
    pub location: ast::Location
}