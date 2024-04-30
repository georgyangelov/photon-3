use std::fmt::{Debug, Display, Formatter};
use crate::frontend::{AST, ASTFunction, ASTLiteral, ASTValue, Pattern, PatternValue};

impl Display for AST {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for ASTValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTValue::Literal(value) => write!(f, "{}", value),

            ASTValue::Block(values) => {
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

            ASTValue::Function(ASTFunction { params, return_type, body }) => {
                write!(f, "(fn [")?;

                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ")?;
                    }

                    match &param.typ {
                        &None => write!(f, "(param {})", param.name)?,
                        &Some(ref typ) => write!(f, "(param {} {})", param.name, typ)?
                    };
                }

                write!(f, "]")?;

                if let Some(typ) = return_type {
                    write!(f, " {}", typ)?;
                }

                write!(f, " {})", body)
            }

            ASTValue::Call { target, name, args } => {
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

            ASTValue::Let { name, value, recursive } => {
                if *recursive {
                    write!(f, "(let-rec {} {})", name, value)
                } else {
                    write!(f, "(let {} {})", name, value)
                }
            }

            ASTValue::NameRef(name) => write!(f, "{}", name),

            ASTValue::FnType { params, return_type } => {
                write!(f, "(fn-type [")?;

                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ")?;
                    }

                    write!(f, "(param {} {})", param.name, param.typ)?;
                }

                write!(f, "] {})", return_type)
            }

            ASTValue::TypeAssert { value, typ } => write!(f, "(type-assert {} {})", value, typ)
        }
    }
}

impl Display for ASTLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTLiteral::Int(value) => write!(f, "{}", value),
            ASTLiteral::Bool(value) => write!(f, "{}", value),
            ASTLiteral::Float(value) => write!(f, "{}", value),
            ASTLiteral::String(value) => {
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