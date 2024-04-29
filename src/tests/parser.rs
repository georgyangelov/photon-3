use crate::frontend::{Lexer, ParseError, Parser};

#[test]
fn parses_number_literals() {
    assert_parse("12345 ", "12345");
    assert_parse("1234.5678 ", "1234.5678")
}

#[test]
fn test_negative_number_literals() {
    assert_parse("-1234", "(- 1234)");
    assert_parse("-1234.5", "(- 1234.5)");
}

#[test]
fn test_constant_object_literals() {
    assert_parse("true", "true");
    assert_parse("false", "false");
}

#[test]
fn test_negating_expressions() {
    assert_parse("-   (5 + 5)", "(- (+ 5 5))");
}

#[test]
fn test_string_literals() {
    assert_parse("\"Hello world!\"", "\"Hello world!\"");
    assert_parse("\"\\\"\\n\"", "\"\\\"\\n\"");
}

fn assert_parse(code: &str, expected: &str) {
    let result = parse(code).expect(format!("Could not parse code {}", code).as_str());

    assert_eq!(result, expected)
}

fn parse(code: &str) -> Result<String, ParseError> {
    let lexer = Lexer::new("<test>", code.chars());
    let mut parser = Parser::new(lexer);
    // let mut result = Vec::new();
    let mut result = String::new();

    while parser.has_next()? {
        let ast = parser.next(false)?;

        // result.push(ast);

        let str = format!("{} ", ast);
        result.push_str(&str);
    }

    result.pop(); // Remove last space

    Ok(result)
}