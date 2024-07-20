use std::fmt::{Display, Formatter};
use crate::ast::*;

impl Display for AST {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Literal(value) => write!(f, "{}", value),

            Value::Block(values) => {
                if values.len() > 0 {
                    write!(f, "{{")?;
                    for ast in values {
                        write!(f, " {}", ast)?;
                    }
                    write!(f, " }}")
                } else {
                    write!(f, "{{}}")
                }
            }

            Value::Function(Function { params, return_type, body }) => {
                write!(f, "(fn [")?;

                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ")?;
                    }

                    let comptime = if param.comptime { "@" } else { "" };

                    match &param.typ {
                        &None => write!(f, "({}param {})", comptime, param.name)?,
                        &Some(ref typ) => write!(f, "({}param {} {})", comptime, param.name, typ)?
                    };
                }

                write!(f, "]")?;

                if let Some(typ) = return_type {
                    write!(f, " {}", typ)?;
                }

                write!(f, " {})", body)
            }

            Value::Call { target, name, args } => {
                write!(f, "({}", name)?;

                if let Some(target) = target {
                    write!(f, " {}", target)?;
                } else {
                    write!(f, " self")?;
                }

                for arg in args {
                    write!(f, " {}", arg)?;
                }

                write!(f, ")")
            }

            Value::Let { name, value, recursive, comptime } => {
                write!(f, "(")?;

                if *comptime {
                    write!(f, "@")?;
                }

                if *recursive {
                    write!(f, "let-rec {} {})", name, value)
                } else {
                    write!(f, "let {} {})", name, value)
                }
            }

            Value::NameRef(name) => write!(f, "{}", name),

            Value::If { condition, on_true, on_false } => {
                write!(f, "(if {} {}", condition, on_true)?;

                if let Some(on_false) = on_false {
                    write!(f, " {}", on_false)?;
                }

                write!(f, ")")
            },

            Value::FnType { params, return_type } => {
                write!(f, "(fn-type [")?;

                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ")?;
                    }

                    write!(f, "(param {} {})", param.name, param.typ)?;
                }

                write!(f, "] {})", return_type)
            }

            Value::TypeAssert { value, typ } => write!(f, "(type-assert {} {})", value, typ),

            Value::CompileTimeExpr(ast) => write!(f, "@{}", ast),
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int(value) => write!(f, "{}", value),
            Literal::Bool(value) => write!(f, "{}", value),
            Literal::Float(value) => write!(f, "{}", value),
            Literal::String(value) => {
                let escaped = value
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\"")
                    .replace("\n", "\\n");

                write!(f, "\"")?;
                write!(f, "{}", escaped)?;
                write!(f, "\"")
            }
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for PatternValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternValue::SpecificValue(value) => write!(f, "{}", value),
            PatternValue::Binding(name) => write!(f, "(val {})", name),

            PatternValue::Call { target, name, args } => {
                write!(f, "<{}", name)?;

                if let Some(target) = target {
                    write!(f, " {}", target)?;
                } else {
                    write!(f, " self")?;
                }

                for arg in args {
                    write!(f, " {}", arg)?;
                }

                write!(f, ">")
            }

            PatternValue::FunctionType { params, return_type } => {
                write!(f, "(fn-type [")?;

                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ")?;
                    }

                    write!(f, "(param {} {})", param.name, param.typ)?;
                }

                write!(f, "] {})", return_type)
            }
        }
    }
}