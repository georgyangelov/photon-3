use crate::frontend::{Lexer, TokenValue};
use crate::frontend::TokenValue::*;

#[test]
fn lexes() {
    assert_eq!(lex("val a = 42.3"), vec![
        Val, Name("a".into()), Equal, DecimalLiteral("42.3".into())
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