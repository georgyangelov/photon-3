use std::collections::VecDeque;
use crate::ast::{Token, Lexer, LexerError};

pub struct LookaheadTokenIterator<I: Iterator<Item = char>> {
    pub lexer: Lexer<I>,
    buffer: VecDeque<Token>
}

impl <I: Iterator<Item = char>> LookaheadTokenIterator<I> {
    pub fn new(lexer: Lexer<I>) -> Self {
        Self {
            lexer,
            buffer: VecDeque::new()
        }
    }

    pub fn next(&mut self) -> Result<Token, LexerError> {
        match self.buffer.pop_front() {
            Some(token) => Ok(token),
            None => self.lexer.next()
        }
    }

    pub fn look_ahead(&mut self) -> LookaheadIteratorIterator<I> {
        LookaheadIteratorIterator {
            lookahead: self,
            i: 0,
        }
    }

    pub fn peek(&mut self) -> Result<&Token, LexerError> {
        if self.buffer.len() == 0 {
            let token = self.lexer.next()?;

            self.buffer.push_back(token);
        }

        Ok(self.buffer.front().unwrap())
    }
}

pub struct LookaheadIteratorIterator<'a, I: Iterator<Item = char>> {
    lookahead: &'a mut LookaheadTokenIterator<I>,
    i: usize
}

impl <'a, I: Iterator<Item = char>> LookaheadIteratorIterator<'a, I> {
    pub fn next(&mut self) -> Result<&Token, LexerError> {
        if self.lookahead.buffer.len() <= self.i {
            let token = self.lookahead.lexer.next()?;

            self.lookahead.buffer.push_back(token);
        }

        let token = self.lookahead.buffer.get(self.i).unwrap();

        self.i += 1;

        Ok(token)
    }
}