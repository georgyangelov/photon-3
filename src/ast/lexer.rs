use std::rc::Rc;
use crate::ast::location::{Location, Position};
use crate::ast::TokenValue::*;

#[derive(Debug, Clone)]
pub struct Token {
    pub value: TokenValue,
    pub location: Location,
    pub whitespace_before: bool
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenValue {
    EOF, NewLine,
    OpenParen, CloseParen, OpenBrace, CloseBrace, OpenBracket, CloseBracket,
    Comma, Dot, At, Colon,
    Val, Recursive, If, Else, Then,
    Equal, Plus, Minus, Asterisk, Slash, LessThan, GreaterThan,
    EqualEqual, PlusEqual, MinusEqual, AsteriskEqual, SlashEqual, LessThanEqual, GreaterThanEqual, NotEqual,
    Not, And, Or,
    Name(Box<str>), IntLiteral(Box<str>), DecimalLiteral(Box<str>), StringLiteral(Box<str>), BoolLiteral(bool)
}

#[derive(Debug)]
pub enum LexerError {
    UnclosedStringLiteral(Position),
    UnexpectedToken(char, Position)
}

pub struct Lexer<I: Iterator<Item = char>> {
    chars: I,

    pub file: Rc<str>,

    c: char,
    pub position: Position,

    next_c: char,
    next_position: Position,

    at_start: bool,
    had_newline: bool,
}

const EOF: char = '\0';

impl <I: Iterator<Item = char>> Lexer<I> {
    pub fn new(file: &str, chars: I) -> Self {
        Lexer {
            chars,

            file: Rc::from(file),

            c: EOF,
            position: Position { line: 0, column: -1 },

            next_c: EOF,
            next_position: Position { line: 0, column: 0 },

            at_start: true,
            had_newline: false,
        }
    }

    pub fn next(&mut self) -> Result<Token, LexerError> {
        if self.at_start {
            self.init_read();
            self.advance();
        }

        let had_whitespace = self.skip_whitespace_and_comments() || self.had_newline;
        self.had_newline = false;

        if self.c == EOF {
            return Ok(Token {
                value: TokenValue::EOF,
                location: Location { file: self.file.clone(), from: self.position, to: self.position },
                whitespace_before: had_whitespace
            })
        }

        match (self.c, self.next_c) {
            ('\n', _) |
            (';', _) => {
                self.had_newline = true;
                self.one_char_token(NewLine, had_whitespace)
            },

            ('(', _) => self.one_char_token(OpenParen, had_whitespace),
            (')', _) => self.one_char_token(CloseParen, had_whitespace),
            ('[', _) => self.one_char_token(OpenBracket, had_whitespace),
            (']', _) => self.one_char_token(CloseBracket, had_whitespace),
            ('{', _) => self.one_char_token(OpenBrace, had_whitespace),
            ('}', _) => self.one_char_token(CloseBrace, had_whitespace),
            (',', _) => self.one_char_token(Comma, had_whitespace),
            ('.', _) => self.one_char_token(Dot, had_whitespace),
            ('@', _) => self.one_char_token(At, had_whitespace),
            (':', _) => self.one_char_token(Colon, had_whitespace),

            ('=', '=') => self.two_char_token(EqualEqual, had_whitespace),
            ('+', '=') => self.two_char_token(PlusEqual, had_whitespace),
            ('-', '=') => self.two_char_token(MinusEqual, had_whitespace),
            ('*', '=') => self.two_char_token(AsteriskEqual, had_whitespace),
            ('/', '=') => self.two_char_token(SlashEqual, had_whitespace),
            ('<', '=') => self.two_char_token(LessThanEqual, had_whitespace),
            ('>', '=') => self.two_char_token(GreaterThanEqual, had_whitespace),
            ('!', '=') => self.two_char_token(NotEqual, had_whitespace),

            ('=', _) => self.one_char_token(Equal, had_whitespace),
            ('+', _) => self.one_char_token(Plus, had_whitespace),
            ('-', _) => self.one_char_token(Minus, had_whitespace),
            ('*', _) => self.one_char_token(Asterisk, had_whitespace),
            ('/', _) => self.one_char_token(Slash, had_whitespace),
            ('<', _) => self.one_char_token(LessThan, had_whitespace),
            ('>', _) => self.one_char_token(GreaterThan, had_whitespace),
            ('!', _) => self.one_char_token(Not, had_whitespace),

            ('"', _) => self.read_string(had_whitespace),

            (c, _) if c.is_digit(10) => self.read_number(had_whitespace),
            (c, _) if is_start_of_name(c) => self.read_atom(had_whitespace),

            (c, _) => Err(LexerError::UnexpectedToken(c, self.position))
        }
    }

    fn read_string(&mut self, had_whitespace: bool) -> Result<Token, LexerError> {
        let mut string = String::new();
        let from = self.position;

        self.advance(); // "

        let mut in_escape_sequence = false;

        while in_escape_sequence || self.c != '"' {
            if self.c == EOF {
                return Err(LexerError::UnclosedStringLiteral(from));
            } else if in_escape_sequence {
                string.push(match self.c {
                    'r' => '\r',
                    'n' => '\n',
                    't' => '\t',
                    '0' => '\0',
                    c => c
                });
                in_escape_sequence = false;
            } else if self.c == '\\' {
                in_escape_sequence = true;
            } else {
                string.push(self.c);
            }

            self.advance();
        }

        self.advance(); // "

        Ok(Token {
            value: StringLiteral(string.into()),
            location: Location { file: self.file.clone(), from, to: self.next_position },
            whitespace_before: had_whitespace
        })
    }

    fn read_number(&mut self, had_whitespace: bool) -> Result<Token, LexerError> {
        let from = self.position;
        let mut string = String::new();
        let mut is_decimal = false;

        while self.c.is_digit(10) || self.c == '.' {
            if self.c == '.' {
                if is_decimal || !self.next_c.is_digit(10) {
                    break
                }

                is_decimal = true;
            }

            string.push(self.c);
            self.advance();
        }

        Ok(Token {
            value: if is_decimal { DecimalLiteral(string.into()) } else { IntLiteral(string.into()) },
            location: Location { file: self.file.clone(), from, to: self.next_position },
            whitespace_before: had_whitespace
        })
    }

    fn read_atom(&mut self, had_whitespace: bool) -> Result<Token, LexerError> {
        let from = self.position;
        let mut string = String::new();

        loop {
            string.push(self.c);
            self.advance();

            if !is_part_of_name(self.c) {
                break;
            }
        }

        if self.c == '!' || self.c == '?' {
            string.push(self.c);
            self.advance();
        }

        let value = match string.as_str() {
            "val" => Val,
            "rec" => Recursive,
            "if" => If,
            "else" => Else,
            "then" => Then,
            "and" => And,
            "or" => Or,
            "true" => BoolLiteral(true),
            "false" => BoolLiteral(false),
            _ => Name(string.into())
        };

        Ok(Token {
            value,
            location: Location { file: self.file.clone(), from, to: self.next_position },
            whitespace_before: had_whitespace
        })
    }

    fn one_char_token(&mut self, value: TokenValue, had_whitespace: bool) -> Result<Token, LexerError> {
        self.advance();

        Ok(Token {
            value,
            location: Location { file: self.file.clone(), from: self.position, to: self.next_position },
            whitespace_before: had_whitespace
        })
    }

    fn two_char_token(&mut self, value: TokenValue, had_whitespace: bool) -> Result<Token, LexerError> {
        let from = self.position;

        self.advance();
        self.advance();

        Ok(Token {
            value,
            location: Location { file: self.file.clone(), from, to: self.next_position },
            whitespace_before: had_whitespace
        })
    }

    fn skip_whitespace_and_comments(&mut self) -> bool {
        let mut in_comment = false;
        let mut had_whitespace = false;

        loop {
            if self.c == EOF {
                break;
            } else if in_comment {
                if self.c == '\n' {
                    in_comment = false;
                }
            } else if self.c == '#' && self.next_c.is_whitespace() {
                in_comment = true;
            } else if !self.c.is_whitespace() || self.c == '\n' {
                break;
            }

            self.advance();
            had_whitespace = true
        }

        had_whitespace
    }

    fn init_read(&mut self) {
        self.at_start = false;

        match self.chars.next() {
            None => (),
            Some(c) => {
                self.next_c = c;
            }
        }
    }

    fn advance(&mut self) -> char {
        let old_c = self.c;

        match self.chars.next() {
            None => {
                self.c = self.next_c;
                self.next_c = EOF;
            },
            Some(c) => {
                self.c = self.next_c;
                self.position = self.next_position;

                self.next_c = c;

                if c == '\n' {
                    self.next_position = Position {
                        line: self.next_position.line + 1,
                        column: 0
                    }
                } else {
                    self.next_position = Position {
                        line: self.next_position.line,
                        column: self.next_position.column + 1
                    }
                }
            }
        }

        old_c
    }
}

fn is_start_of_name(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

fn is_part_of_name(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}