use crate::ast::{Lexer, TokenValue};
use crate::ast::TokenValue::{DecimalLiteral, EOF, Equal, Name, Val};

#[test]
fn lexes_some_tokens() {
    assert_eq!(lex("val a = 42.3"), vec![
        Val, Name("a".into()), Equal, DecimalLiteral("42.3".into())
    ]);
}

fn lex(code: &str) -> Vec<TokenValue> {
    let mut lexer = Lexer::new("<test>", code.chars());
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next().expect("Error during lexing");
        if token.value == EOF {
            break;
        }

        tokens.push(token.value);
    }

    tokens
}