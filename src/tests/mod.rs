use crate::parser::{Lexer, TokenValue};
use crate::parser::TokenValue::*;

#[test]
fn lexes() {
    assert_eq!(lex("val a = 42"), vec![
        Val, Name("a".into()), Equal, IntLiteral("42".into())
    ]);
}

fn lex(code: &str) -> Vec<TokenValue> {
    let mut lexer = Lexer::new("<test>", code.chars());
    let mut tokens = Vec::new();

    while let Some(token) = lexer.next().expect("Error during lexing") {
        tokens.push(token.value);
    }

    tokens
}