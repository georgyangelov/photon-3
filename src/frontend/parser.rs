use std::mem::swap;
use crate::frontend::*;
use crate::frontend::lookahead_token_iterator::{LookaheadIteratorIterator, LookaheadTokenIterator};
use crate::frontend::ParseError::UnexpectedPattern;
use crate::frontend::TokenValue::*;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Box<str>, Token),
    UnexpectedEOF,
    UnexpectedPattern(Pattern),
    CouldNotParseNumber(Location),
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

    pub fn read_all_as_block(&mut self) -> Result<AST, ParseError> {
        let mut asts = Vec::new();

        while self.has_next()? {
            let ast = self.next(false)?;

            asts.push(ast);
        }

        let location =
            if asts.len() > 0 {
                asts[0].location.extend(&asts[asts.len() - 1].location)
            } else {
                panic!("Nothing to read")
            };

        Ok(AST {
            value: ASTValue::Block(asts),
            location
        })
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
        let mut left = self.parse_primary(require_call_parens, has_lower_priority_target)?;

        loop {
            if self.newline {
                return Ok(left);
            }

            let precedence = match Self::operator_precedence(&self.t.value) {
                // Not an operator
                None => return Ok(left),
                Some(precedence) => precedence
            };

            if precedence < min_precedence {
                return Ok(left)
            }

            let operator = self.read()?; // <operator>

            let right = self.parse_expression(precedence + 1, require_call_parens, has_lower_priority_target)?;

            let left_ast = Self::assert_ast(left)?;

            left = if operator.value == Colon {
                let right = Self::assert_ast(right)?;
                let location = left_ast.location.extend(&right.location);

                ASTOrPattern::AST(AST {
                    value: ASTValue::TypeAssert {
                        value: Box::new(left_ast),
                        typ: Box::new(right),
                    },
                    location
                })
            } else if let ASTOrPattern::Pattern(right) = right {
                let method_name = Self::binary_operator_method_name(&operator.value)
                    .expect(format!("Operator name not found: {:?}", operator.value.clone()).as_str());

                let location = left_ast.location.extend(&right.location);

                ASTOrPattern::Pattern(Pattern {
                    value: PatternValue::Call {
                        target: Some(left_ast.into()),
                        name: method_name.into(),
                        args: [right].into()
                    },
                    location
                })
            } else {
                let method_name = Self::binary_operator_method_name(&operator.value)
                    .expect(format!("Operator name not found: {:?}", operator.value.clone()).as_str());

                let right = Self::assert_ast(right)?;
                let location = left_ast.location.extend(&right.location);

                ASTOrPattern::AST(AST {
                    value: ASTValue::Call {
                        target: Some(left_ast.into()),
                        name: method_name.into(),
                        args: [right].into()
                    },
                    location
                })
            }
        }
    }

    fn parse_primary(&mut self, require_call_parens: bool, has_lower_priority_target: bool) -> Result<ASTOrPattern, ParseError> {
        // TODO: Check if this allows to have `-val a = 42` which should be invalid
        if self.t.value == Recursive || self.t.value == Val {
            let start_loc = self.t.location.clone();

            return self.parse_val(start_loc, false, require_call_parens, has_lower_priority_target);
        }

        if self.t.value == At {
            let at = self.read()?; // @

            if self.t.value == Recursive || self.t.value == Val {
                return self.parse_val(at.location, true, require_call_parens, has_lower_priority_target);
            }

            return self.parse_compile_time_expression(at.location, require_call_parens, has_lower_priority_target)
        }

        if self.t.value == Minus {
            return self.parse_unary_operator(require_call_parens, has_lower_priority_target)
        }

        let mut target = self.parse_call_target(require_call_parens, has_lower_priority_target)?;

        loop {
            let (new_target, has_more) = self.try_to_parse_call(target, require_call_parens, has_lower_priority_target)?;

            if has_more {
                target = new_target
            } else {
                return Ok(new_target)
            }
        }
    }

    fn parse_val(&mut self, start_loc: Location, comptime: bool, require_call_parens: bool, has_lower_priority_target: bool) -> Result<ASTOrPattern, ParseError> {
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
                0,
                require_call_parens,
                has_lower_priority_target
            )?
        )?;

        let value_with_type = match type_ast {
            None => value_ast,
            Some(typ) => {
                let location = typ.location.clone();

                AST {
                    value: ASTValue::TypeAssert { value: Box::new(value_ast), typ: Box::new(typ) },
                    location
                }
            }
        };

        return Ok(ASTOrPattern::AST(AST {
            value: ASTValue::Let {
                name,
                value: Box::new(value_with_type),
                recursive,
                comptime
            },
            location: start_loc.extend(&self.last_location)
        }))
    }

    fn parse_call_target(&mut self, require_call_parens: bool, has_lower_priority_target: bool) -> Result<ASTOrPattern, ParseError> {
        match &self.t.value {
            &BoolLiteral(_) => self.parse_bool(),
            &IntLiteral(_) => self.parse_int(),
            &DecimalLiteral(_) => self.parse_float(),
            &StringLiteral(_) => self.parse_string(),
            &Name(_) => self.parse_name(),

            &OpenBrace => self.parse_lambda_or_lambda_type(has_lower_priority_target),
            &At => {
                let at = self.read()?; // @

                self.parse_compile_time_expression(at.location, require_call_parens, has_lower_priority_target)
            },

            &Not => self.parse_unary_operator(require_call_parens, has_lower_priority_target),

            &OpenParen => self.parse_expression_starting_with_open_paren(has_lower_priority_target),

            _ => Err(ParseError::UnexpectedToken("Unexpected token".into(), self.t.clone()))
        }
    }

    fn parse_compile_time_expression(
        &mut self,
        start_loc: Location,
        require_call_parens: bool,
        has_lower_priority_target: bool
    ) -> Result<ASTOrPattern, ParseError> {
        let expr = self.parse_primary(require_call_parens, has_lower_priority_target)?;
        let expr = Self::assert_ast(expr)?;

        Ok(ASTOrPattern::AST(AST {
            value: ASTValue::CompileTimeExpr(Box::new(expr)),
            location: start_loc.extend(&self.last_location)
        }))
    }

    fn parse_bool(&mut self) -> Result<ASTOrPattern, ParseError> {
        let t = self.read()?; // true / false
        let value = match t.value {
            BoolLiteral(value) => value,
            _ => panic!("Logic error - expected bool literal")
        };

        Ok(ASTOrPattern::AST(AST {
            value: ASTValue::Literal(ASTLiteral::Bool(value)),
            location: t.location
        }))
    }

    fn parse_int(&mut self) -> Result<ASTOrPattern, ParseError> {
        let t = self.read()?; // 1234
        let value = match t.value {
            IntLiteral(value) => value,
            _ => panic!("Logic error - expected int literal")
        };
        let value = value.parse::<i64>()
            .map_err(|_| ParseError::CouldNotParseNumber(t.location.clone()))?;

        Ok(ASTOrPattern::AST(AST {
            value: ASTValue::Literal(ASTLiteral::Int(value)),
            location: t.location
        }))
    }

    fn parse_float(&mut self) -> Result<ASTOrPattern, ParseError> {
        let t = self.read()?; // 1234.45
        let value = match t.value {
            DecimalLiteral(value) => value,
            _ => panic!("Logic error - expected decimal literal")
        };
        let value = value.parse::<f64>()
            .map_err(|_| ParseError::CouldNotParseNumber(t.location.clone()))?;

        Ok(ASTOrPattern::AST(AST {
            value: ASTValue::Literal(ASTLiteral::Float(value)),
            location: t.location
        }))
    }

    fn parse_string(&mut self) -> Result<ASTOrPattern, ParseError> {
        let t = self.read()?; // "string"
        let value = match t.value {
            StringLiteral(value) => value,
            _ => panic!("Logic error - expected string literal")
        };

        Ok(ASTOrPattern::AST(AST {
            value: ASTValue::Literal(ASTLiteral::String(value)),
            location: t.location
        }))
    }

    fn parse_name(&mut self) -> Result<ASTOrPattern, ParseError> {
        let t = self.read()?; // name
        let value = match t.value {
            Name(name) => name,
            _ => panic!("Logic error - expected name")
        };

        Ok(ASTOrPattern::AST(AST {
            value: ASTValue::NameRef(value),
            location: t.location
        }))
    }

    fn parse_unary_operator(&mut self, require_call_parens: bool, has_lower_priority_target: bool) -> Result<ASTOrPattern, ParseError> {
        let start_loc = self.t.location.clone();
        let t = self.read()?; // - / !

        let expression = Self::assert_ast(self.parse_primary(require_call_parens, has_lower_priority_target)?)?;
        let location = start_loc.extend(&expression.location);

        return Ok(ASTOrPattern::AST(AST {
            value: ASTValue::Call {
                target: Some(Box::new(expression)),

                name: Self::unary_operator_method_name(&t.value)
                    .ok_or_else(|| ParseError::UnexpectedToken("Invalid unary operator".into(), t.clone()) )?
                    .into(),

                args: [].into()
            },
            location
        }));
    }

    fn parse_expression_starting_with_open_paren(&mut self, has_lower_priority_target: bool) -> Result<ASTOrPattern, ParseError> {
        {
            let mut lookahead = self.lexer.look_ahead();
            if Self::check_if_open_paren_for_lambda(&mut lookahead)? {
                return self.parse_lambda_or_lambda_type(has_lower_priority_target);
            }
        }

        let start_token = self.read()?; // (
        let start_location = start_token.location;

        let mut values = Vec::new();

        loop {
            let expr = self.parse_expression(0, false, false)?;
            let expr = Self::assert_ast(expr)?;

            values.push(expr);

            if self.t.value == CloseParen || !self.newline {
                break
            }
        }

        if self.t.value != CloseParen {
            return Err(ParseError::UnexpectedToken("Unmatched parentheses or extra expressions. Expected ')'".into(), self.t.clone()))
        }

        let t = self.read()?;

        if values.len() == 1 {
            Ok(ASTOrPattern::AST(values.pop().unwrap()))
        } else {
            Ok(ASTOrPattern::AST(AST {
                value: ASTValue::Block(values.into()),
                location: start_location.extend(&t.location)
            }))
        }
    }

    fn try_to_parse_call(&mut self, target: ASTOrPattern, require_call_parens: bool, has_lower_priority_target: bool) -> Result<(ASTOrPattern, bool), ParseError> {
        // target.call
        if self.t.value == Dot {
            if self.t.whitespace_before && has_lower_priority_target {
                // array.map { 42 } .filter (x) x > 0
                return Ok((target, false));
            }

            let target = Self::assert_ast(target)?;

            return Ok((self.parse_call_with_explicit_target(target, require_call_parens)?, true));
        }

        // name a
        // name(a)
        if matches!(target, ASTOrPattern::AST(AST { value: ASTValue::NameRef(_), ..})) {
            let is_definitely_a_call = self.t.value == OpenParen;

            if !self.current_expression_may_end() && (!require_call_parens || is_definitely_a_call) {
                let (name, name_location) = match target {
                    ASTOrPattern::AST(AST { value: ASTValue::NameRef(name), location }) => (name, location),
                    _ => panic!("Should not happen - target was not an AST")
                };

                let args = self.parse_arguments(require_call_parens, true)?;
                let location = name_location.extend(&self.last_location);

                return Ok((self.build_call_ast_or_pattern(None, name, args, location)?, true));
            }
        }

        // expression(...)
        if self.t.value == OpenParen && !self.t.whitespace_before {
            let args = self.parse_arguments(require_call_parens, false)?;
            let target = Self::assert_ast(target)?;
            let name = "call";
            let location = target.location.extend(&self.last_location);

            return Ok((self.build_call_ast_or_pattern(Some(target), name.into(), args, location)?, true))
        }

        Ok((target, false))
    }

    fn parse_call_with_explicit_target(&mut self, target: AST, require_call_parens: bool) -> Result<ASTOrPattern, ParseError> {
        self.read()?; // .

        let name = self.read()?;
        let name = match name.value {
            Name(string) => string,
            value => match Self::operator_method_name(&value) {
                None => return Err(ParseError::UnexpectedToken("Expected a valid method name".into(), self.t.clone())),
                Some(name) => name
            }.into()
        };

        let args = self.parse_arguments(require_call_parens, true)?;
        let location = target.location.extend(&self.last_location);

        self.build_call_ast_or_pattern(Some(target), name, args, location)
    }

    fn build_call_ast_or_pattern(&self, target: Option<AST>, name: Box<str>, args: Vec<ASTOrPattern>, location: Location) -> Result<ASTOrPattern, ParseError> {
        let is_pattern = args.iter().any(|arg| matches!(arg, ASTOrPattern::Pattern(_)));

        if is_pattern {
            Ok(ASTOrPattern::Pattern(Pattern {
                value: PatternValue::Call {
                    target: target.map(Box::new),
                    name,
                    args: args.into_iter().map(|value| Self::coerce_to_pattern(value)).collect(),
                },
                location
            }))
        } else {
            let mut ast_args = Vec::with_capacity(args.len());
            for arg in args {
                ast_args.push(Self::assert_ast(arg)?)
            }

            Ok(ASTOrPattern::AST(AST {
                value: ASTValue::Call {
                    target: target.map(Box::new),
                    name,
                    args: ast_args,
                },
                location
            }))
        }
    }

    fn parse_arguments(&mut self, require_parens: bool, has_lower_priority_target: bool) -> Result<Vec<ASTOrPattern>, ParseError> {
        let with_parens =
            if self.t.value == OpenParen && !self.t.whitespace_before {
                self.read()?; // (
                true
            } else {
                false
            };

        let mut args = Vec::new();

        if !with_parens && self.current_expression_may_end() {
            return Ok(args);
        }

        if !with_parens && require_parens {
            return Ok(args);
        }

        if with_parens && self.t.value == CloseParen {
            self.read()?; // )
            return Ok(args);
        }

        let expr = self.parse_expression(0, false, has_lower_priority_target && !with_parens)?;
        args.push(expr);
        while self.t.value == Comma {
            self.read()?; // ,

            let expr = self.parse_expression(0, false, has_lower_priority_target && !with_parens)?;
            args.push(expr);
        }

        if with_parens {
            if self.t.value != CloseParen {
                return Err(ParseError::UnexpectedToken("Unexpected ')'".into(), self.t.clone()));
            }

            self.read()?; // )
        } else if !self.current_expression_may_end() {
            return Err(ParseError::UnexpectedToken("Expected current expression to end (either new line, ';' or ')')".into(), self.t.clone()));
        }

        Ok(args)
    }

    fn parse_lambda_or_lambda_type(&mut self, has_lower_priority_target: bool) -> Result<ASTOrPattern, ParseError> {
        // This aims to fix parse of lambdas using only braces on a separate line, e.g. `{ a }`
        // Since there was a newline before, but we don't care
        self.newline = false;

        let start_location = self.last_location.clone();

        let has_param_parens = self.t.value == OpenParen;
        let params = if has_param_parens {
            self.parse_lambda_params()?
        } else {
            Vec::new()
        };

        let has_return_type = self.t.value == Colon;
        let return_type = if has_return_type {
            self.read()?; // :

            Some(self.parse_expression(0, true, false)?)
        } else {
            None
        };

        let has_body = !self.current_expression_may_end();

        if has_body {
            // It's a function

            let has_block = self.t.value == OpenBrace;
            let body = if has_block {
                self.read()?; // {

                let block = self.parse_block()?;

                if self.t.value != CloseBrace {
                    return Err(ParseError::UnexpectedToken("Expected '}'".into(), self.t.clone()));
                }
                self.read()?; // }

                block
            } else {
                let expr = self.parse_expression(0, false, has_lower_priority_target)?;

                Self::assert_ast(expr)?
            };

            let return_type = match return_type {
                None => None,
                Some(typ) => Some(Box::new(Self::assert_ast(typ)?))
            };

            Ok(ASTOrPattern::AST(AST {
                value: ASTValue::Function(ASTFunction {
                    params,
                    body: Box::new(body),
                    return_type
                }),
                location: start_location.extend(&self.last_location)
            }))
        } else {
            // It's a function type

            let returns = match return_type {
                None => return Err(ParseError::UnexpectedToken("Function types need to have explicit return type".into(), self.t.clone())),
                Some(typ) => typ
            };

            let location = start_location.extend(&self.t.location);

            let mut has_pattern_params = false;
            for param in &params {
                match param {
                    &ASTParam {
                        typ: Some(Pattern { value: PatternValue::SpecificValue(_), .. }),
                        ..
                    } => {},
                    &ASTParam { typ: Some(_), .. } => has_pattern_params = true,
                    _ => return Err(ParseError::UnexpectedToken("Function types need to have explicit parameter types".into(), self.t.clone()))
                }
            }

            let has_return_type_pattern = matches!(returns, ASTOrPattern::Pattern(_));

            if has_pattern_params || has_return_type_pattern {
                let params = params.into_iter().map(|it|
                    PatternParam {
                        name: it.name,
                        typ: it.typ.expect("Logic error: param did not have a type"),
                        location: it.location
                    }
                ).collect();
                let return_type = Box::new(Self::coerce_to_pattern(returns));

                // Function type pattern, e.g. `(a: val T): T`
                Ok(ASTOrPattern::Pattern(Pattern {
                    value: PatternValue::FunctionType { params, return_type },
                    location
                }))
            } else {
                let mut ast_params = Vec::with_capacity(params.len());
                for it in params {
                    ast_params.push(ASTTypeParam {
                        name: it.name,
                        typ: match it.typ {
                            Some(Pattern { value: PatternValue::SpecificValue(ast), .. }) => ast,
                            None => return Err(ParseError::UnexpectedToken("Function types need to have explicit parameter types".into(), self.t.clone())),
                            Some(_) => return Err(ParseError::UnexpectedToken("Function types cannot have patterns as param types".into(), self.t.clone()))
                        },
                        location: it.location
                    });
                }
                let return_type = Box::new(Self::assert_ast(returns)?);

                // Function type, e.g. `(a: Int): Int`
                Ok(ASTOrPattern::AST(AST {
                    value: ASTValue::FnType { params: ast_params, return_type },
                    location
                }))
            }
        }
    }

    fn parse_lambda_params(&mut self) -> Result<Vec<ASTParam>, ParseError> {
        let mut params = Vec::new();

        self.read()?; // (

        let has_params = self.t.value != CloseParen;
        if has_params {
            loop {
                let (name, name_location) = self.read_name("Expected parameter name")?;

                let typ = if self.t.value == Colon {
                    self.read()?; // :

                    let expr = self.parse_expression(0, true, false)?;

                    Some(Self::coerce_to_pattern(expr))
                } else {
                    None
                };

                params.push(ASTParam {
                    name,
                    typ,
                    location: name_location.extend(&self.last_location)
                });

                if self.t.value != Comma { break }
                self.read()?; // ,
            }
        }

        if self.t.value != CloseParen {
            return Err(ParseError::UnexpectedToken("Expected ')'".into(), self.t.clone()));
        }
        self.read()?; // )

        Ok(params)
    }

    fn parse_block(&mut self) -> Result<AST, ParseError> {
        let mut values = Vec::new();
        let start_location = self.t.location.clone();

        while self.t.value != CloseBrace {
            let expr = self.parse_expression(0, false, false)?;
            let ast = Self::assert_ast(expr)?;

            values.push(ast);
        }

        if values.len() == 1 {
            Ok(values.into_iter().next().unwrap())
        } else {
            Ok(AST {
                value: ASTValue::Block(values),
                location: start_location.extend(&self.t.location)
            })
        }
    }

    fn current_expression_may_end(&self) -> bool {
        self.newline || match &self.t.value {
            EOF => true,

            // Binary operators
            Plus | Minus | Asterisk | Slash | LessThan | GreaterThan | EqualEqual | PlusEqual |
            MinusEqual | AsteriskEqual | SlashEqual | LessThanEqual | GreaterThanEqual |
            NotEqual | And | Or => true,

            Colon | Comma | CloseParen | Dot | CloseBracket | CloseBrace => true,

            // This is because of lambda types
            Equal => true,

            _ => false
        }
    }

    fn check_if_open_paren_for_lambda(reader: &mut LookaheadIteratorIterator<I>) -> Result<bool, ParseError> {
        let mut nested_paren_level = 1;
        let mut token = reader.next().map_err(ParseError::LexerError)?; // (

        while nested_paren_level > 0 {
            if token.value == EOF {
                return Ok(false);
            }

            match token.value {
                OpenParen => nested_paren_level += 1,
                CloseParen => nested_paren_level -= 1,
                _ => ()
            }

            token = reader.next().map_err(ParseError::LexerError)?;
        }

        Ok(match token.value {
            EOF => false,
            NewLine => false,

            // This is usually ambiguous, that's why we need to recursively check:
            // (thisIsAFunction)(42) -> expr
            // (function.call + something)(42) -> expr
            // (fnVar) (42 + argument) -> expr
            // (a) (b) a + b -> lambda
            OpenParen => Self::check_if_open_paren_for_lambda(reader)?,

            CloseParen => false,
            OpenBrace => true,
            CloseBrace => false,
            OpenBracket => true,
            CloseBracket => false,
            Comma => false,
            Dot => false,
            At => false,
            Equal => false,

            // This is for return type of lambdas. For example:
            // (1 + 2): Int
            // (a: Int): Int { a + 42 }
            Colon => true,

            Val => false,
            Recursive => false,

            // Binary operators - these suggest that the parens were for an expression, not a lambda
            Plus => false,
            Minus => false,
            Asterisk => false,
            Slash => false,
            LessThan => false,
            GreaterThan => false,
            EqualEqual => false,
            PlusEqual => false,
            MinusEqual => false,
            AsteriskEqual => false,
            SlashEqual => false,
            LessThanEqual => false,
            GreaterThanEqual => false,
            NotEqual => false,
            And => false,
            Or => false,

            // TODO: Is this correct?
            Not => true,

            Name(_) => true,
            IntLiteral(_) => true,
            DecimalLiteral(_) => true,
            StringLiteral(_) => true,
            BoolLiteral(_) => true
        })
    }

    fn unary_operator_method_name(operator: &TokenValue) -> Option<&'static str> {
        match operator {
            // TODO: These should probably be prefixed with `@` to indicate unary operator
            &Not => Some("!"),
            &Minus => Some("-"),

            _ => None
        }
    }

    fn binary_operator_method_name(operator: &TokenValue) -> Option<&'static str> {
        Some(match operator {
            &Plus => "+",
            &Minus => "-",
            &Asterisk => "*",
            &Slash => "/",
            &LessThan => "<",
            &GreaterThan => ">",
            &EqualEqual => "==",
            &PlusEqual => "+=",
            &MinusEqual => "-=",
            &AsteriskEqual => "*=",
            &SlashEqual => "/=",
            &LessThanEqual => "<=",
            &GreaterThanEqual => ">=",
            &NotEqual => "!=",
            &And => "and",
            &Or => "or",

            _ => return None
        })
    }

    fn operator_method_name(token: &TokenValue) -> Option<&'static str> {
        Self::binary_operator_method_name(token).or_else(|| Self::unary_operator_method_name(token))
    }

    fn operator_precedence(token: &TokenValue) -> Option<i8> {
        // TODO: Handle *=, /=, +=, -=
        match token {
            // Equal => Some(1),
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
            ASTOrPattern::AST(ast) => Ok(ast),
            ASTOrPattern::Pattern(pattern) => Err(UnexpectedPattern(pattern))
        }
    }

    fn coerce_to_pattern(value: ASTOrPattern) -> Pattern {
        match value {
            ASTOrPattern::AST(ast) => {
                let location = ast.location.clone();

                Pattern {
                    value: PatternValue::SpecificValue(ast),
                    location
                }
            },
            ASTOrPattern::Pattern(pattern) => pattern
        }
    }

    fn expect_token(&mut self, value: TokenValue, message: &str) -> Result<(), ParseError> {
        if self.t.value == value {
            Ok(())
        } else {
            return Err(ParseError::UnexpectedToken(message.into(), self.t.clone()))
        }
    }

    fn read_name(&mut self, message: &str) -> Result<(Box<str>, Location), ParseError> {
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
                Err(lex_error) => return Err(ParseError::LexerError(lex_error))
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

    // fn peek(&mut self) -> Result<&Token, ParseError> {
    //     match self.lexer.peek() {
    //         Ok(token) => Ok(token),
    //         Err(lexError) => Err(ParseError::LexerError(lexError))
    //     }
    // }
}