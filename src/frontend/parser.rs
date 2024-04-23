use std::mem::swap;
use std::num::ParseIntError;
use std::ptr::null;
use crate::frontend::*;
use crate::frontend::lookahead_token_iterator::LookaheadTokenIterator;
use crate::frontend::ParseError::UnexpectedPattern;
use crate::frontend::TokenValue::*;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Box<str>, Token),
    UnexpectedEOF,
    UnexpectedPattern(Pattern),
    LexerError(LexerError)
}

enum ASTOrPattern {
    AST(AST),
    Pattern(Pattern)
}

pub struct Parser<I: Iterator<Item = char>> {
    lexer: LookaheadTokenIterator<I>,

    at_start: bool,
    newline: bool,
    t: Token,
    last_location: Location
}

impl <I: Iterator<Item = char>> Parser<I> {
    pub fn new(lexer: Lexer<I>) -> Self {
        let null_location = Location {
            file: "".into(),
            from: Position { line: -1, column: -1 },
            to: Position { line: -1, column: -1 }
        };

        Parser {
            lexer: LookaheadTokenIterator::new(lexer),
            at_start: true,
            newline: false,
            t: Token {
                value: EOF,
                location: null_location.clone(),
                whitespace_before: false,
            },
            last_location: null_location
        }
    }

    pub fn has_next(&mut self) -> Result<bool, ParseError> {
        if self.at_start { self.read()?; }

        Ok(self.t.value != EOF)
    }

    pub fn next(&mut self, require_call_parens: bool) -> Result<AST, ParseError> {
        if self.at_start { self.read()?; }

        let expr = self.parse_expression(0, require_call_parens, false)?;

        Self::assert_ast(expr)
    }

    fn parse_expression(
        &mut self,
        min_precedence: i8,
        require_call_parens: bool,
        has_lower_priority_target: bool
    ) -> Result<ASTOrPattern, ParseError> {

    }

    fn parse_primary(&mut self, require_call_parens: bool, has_lower_priority_target: bool) -> Result<ASTOrPattern, ParseError> {
        if self.t.value == Recursive || self.t.value == Val {
            return self.parse_val(require_call_parens, has_lower_priority_target);
        }

        if self.t.value == Minus {
            let start_loc = &self.t.location;
            self.read()?; // -

            let expression = Self::assert_ast(self.parse_primary(require_call_parens, has_lower_priority_target)?)?;
            let location = start_loc.extend(&expression.location);

            return Ok(ASTOrPattern::AST(AST {
                value: ASTValue::Call {
                    target: expression,

                    // TODO: This should probably be `@-`, indicating unary minus
                    name: "-".into(),

                    args: [].into(),
                    maybe_var_call: false,
                },
                location
            }));
        }

        let mut target = self.parse_call_target(require_call_parens, has_lower_priority_target)?;

        loop {
            let new_target = self.try_to_parse_call(target, require_call_parens, has_lower_priority_target)?;

            match new_target {
                None => return Ok(target),
                Some(t) => target = t,
            }
        }
    }

    fn parse_val(&mut self, require_call_parens: bool, has_lower_priority_target: bool) -> Result<ASTOrPattern, ParseError> {
        let start_loc = &self.t.location;
        let recursive = self.t.value == Recursive;
        if recursive {
            self.read()?; // rec

            self.expect_token(Val, "rec must be followed by val")?;
        }

        self.read()?; // val

        let (name, name_loc) = self.read_name("val must be followed by a name")?;

        let is_assignment = self.t.value == Colon || self.t.value == Equal;
        if !is_assignment {
            if recursive {
                return Err(ParseError::UnexpectedToken("rec vals must be assignments".into(), self.read().unwrap()))
            }

            return Ok(ASTOrPattern::Pattern(Pattern {
                value: PatternValue::Binding(name),
                location: start_loc.extend(&name_loc)
            }))
        }

        let type_ast = if self.t.value == Colon {
            self.read()?; // :

            let expr = self.parse_expression(0, true, false)?;

            Some(Self::assert_ast(expr)?)
        } else { None };

        self.expect_token(Equal, "val needs to have an =")?;
        self.read()?; // =

        let value_ast = Self::assert_ast(
            self.parse_expression(
                Self::operator_precedence(&Equal).unwrap() + 1,
                require_call_parens,
                has_lower_priority_target
            )?
        )?;

        let value_with_type = match type_ast {
            None => value_ast,
            Some(typ) => AST {
                value: ASTValue::TypeAssert { value: value_ast, typ },
                location: typ.location.clone()
            }
        };

        return Ok(ASTOrPattern::AST(AST {
            value: ASTValue::Let {
                name,
                value: value_with_type,
                recursive
            },
            location: start_loc.extend(&self.last_location)
        }))
    }

    fn parse_call_target(&mut self, require_call_parens: bool, has_lower_priority_target: bool) -> Result<ASTOrPattern, ParseError> {


        let t = self.read()?;

        match t.value {
            BoolLiteral(value) => {
                Ok(ASTOrPattern::AST(AST {
                    value: ASTValue::Literal(ASTLiteral::Bool(value)),
                    location: t.location
                }))
            },

            IntLiteral(value) => {
                let int_value = match value.parse::<i64>() {
                    Ok(value) => value,
                    Err(_) => return Err(ParseError::UnexpectedToken("Invalid int".into(), t.clone()))
                };

                Ok(ASTOrPattern::AST(AST {
                    value: ASTValue::Literal(ASTLiteral::Int(int_value)),
                    location: t.location
                }))
            },

            DecimalLiteral(value) => {
                let float_value = match value.parse::<f64>() {
                    Ok(value) => value,
                    Err(_) => return Err(ParseError::UnexpectedToken("Invalid float".into(), t.clone()))
                };

                Ok(ASTOrPattern::AST(AST {
                    value: ASTValue::Literal(ASTLiteral::Float(float_value)),
                    location: t.location
                }))
            },

            StringLiteral(string) => {
                Ok(ASTOrPattern::AST(AST {
                    value: ASTValue::Literal(ASTLiteral::String(string)),
                    location: t.location
                }))
            },

            Name(string) => {
                Ok(ASTOrPattern::AST(AST {
                    value: ASTValue::NameRef(string),
                    location: t.location
                }))
            },

            OpenBrace => self.parse_lambda_or_lambda_type(has_lower_priority_target, false)?
        }
    }

    fn operator_precedence(token: &TokenValue) -> Option<i8> {
        match token {
            Equal => Some(1),
            Or => Some(2),
            And => Some(3),
            EqualEqual | LessThan | GreaterThan | LessThanEqual | GreaterThanEqual | NotEqual => Some(4),
            Plus | Minus => Some(5),
            Asterisk | Slash => Some(6),
            Colon => Some(7),
            _ => None
        }
    }

    fn assert_ast(value: ASTOrPattern) -> Result<AST, ParseError> {
        match value {
            Ok(ASTOrPattern::AST(ast)) => Ok(ast),
            Ok(ASTOrPattern::Pattern(pattern)) => Err(UnexpectedPattern(pattern)),
            Err(error) => Err(error)
        }
    }

    fn expect_token(&mut self, value: TokenValue, message: &str) -> Result<(), ParseError> {
        if self.t.value == value {
            Ok(())
        } else {
            return Err(ParseError::UnexpectedToken(message.into(), self.t.clone()))
        }
    }

    fn read_name(&mut self, message: &str) -> Result<(Name, Location), ParseError> {
        let t = self.read()?;

        match t {
            Token { value: Name(name), location, .. } => Ok((name, location)),
            _ => Err(ParseError::UnexpectedToken(message.into(), t))
        }
    }

    fn read(&mut self) -> Result<Token, ParseError> {
        self.newline = false;

        let old_token = loop {
            let mut token = match self.lexer.next() {
                Ok(token) => token,
                Err(lexError) => return Err(ParseError::LexerError(lexError))
            };

            if token.value == NewLine {
                self.newline = true;
                continue;
            } else {
                swap(&mut self.t, &mut token);

                break token;
            }
        };

        if self.at_start {
            self.at_start = false;
            self.last_location = Location {
                file: self.lexer.lexer.file.clone(),
                from: self.lexer.lexer.position,
                to: self.lexer.lexer.position
            };
        } else {
            self.last_location = old_token.location.clone();
        }

        Ok(old_token)
    }

    fn peek(&mut self) -> Result<&Token, ParseError> {
        match self.lexer.peek() {
            Ok(token) => Ok(token),
            Err(lexError) => Err(ParseError::LexerError(lexError))
        }
    }
}