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

#[test]
fn test_infix_operators() {
    assert_parse("1 + 2 + 3 + 4", "(+ (+ (+ 1 2) 3) 4)");
    assert_parse("1 * 2 * 3 * 4", "(* (* (* 1 2) 3) 4)");
    assert_parse("1 - 2 - 3 - 4", "(- (- (- 1 2) 3) 4)");
    assert_parse("1 / 2 / 3 / 4", "(/ (/ (/ 1 2) 3) 4)");
    assert_parse("1 or 2 and 3 or 4", "(or (or 1 (and 2 3)) 4)");
}

#[test]
fn test_assignment() {
    assert_parse("val a = 15", "(let a 15)");
    assert_parse("val a = 5 * 5", "(let a (* 5 5))");
}

#[test]
fn test_recursive_assignment() {
    assert_parse("rec val a = 15", "(let-rec a 15)");
    assert_parse("rec val a = 5 * 5", "(let-rec a (* 5 5))");
    assert_parse("rec val a = 1; a * 2; rec val b = 2; a + b", "(let-rec a 1) (* a 2) (let-rec b 2) (+ a b)");
}

#[test]
fn test_prefix_operators() {
    assert_parse("!a", "(! a)");
    assert_parse("!!a", "(! (! a))");
}

#[test]
fn test_operator_precedence() {
    assert_parse("1 + 2 * 3", "(+ 1 (* 2 3))");

    assert_parse("1 == 2 + 2 * 3", "(== 1 (+ 2 (* 2 3)))");
    assert_parse("1 == 2 * 2 + 3", "(== 1 (+ (* 2 2) 3))");
    assert_parse("1 != 2 * 2 + 3", "(!= 1 (+ (* 2 2) 3))");
    assert_parse("1 <= 2 * 2 + 3", "(<= 1 (+ (* 2 2) 3))");
    assert_parse("1 >= 2 * 2 + 3", "(>= 1 (+ (* 2 2) 3))");
    assert_parse("1 < 2 * 2 + 3", "(< 1 (+ (* 2 2) 3))");
    assert_parse("1 > 2 * 2 + 3", "(> 1 (+ (* 2 2) 3))");

    assert_parse("1 - 2 / 3 * 4 + 5", "(+ (- 1 (* (/ 2 3) 4)) 5)");
}

#[test]
fn test_parens_for_precedence() {
    assert_parse("(1 + 2) * 3", "(* (+ 1 2) 3)");
}

#[test]
fn test_unary_operator_precedence() {
    assert_parse("1 + 2 * !a", "(+ 1 (* 2 (! a)))");
    assert_parse("!1 + 2 * a", "(+ (! 1) (* 2 a))");
}

#[test]
fn test_newlines_in_expressions() {
    assert_parse("val a =\n\n 5 * 5", "(let a (* 5 5))");

    assert_parse("1 + 2 - 5", "(- (+ 1 2) 5)");
    assert_parse("1 + 2 \n - 5", "(+ 1 2) (- 5)");

    assert_parse("1 +\n\n 2 * \n\n 3", "(+ 1 (* 2 3))");

//    TODO
//    parse_error("1 +\n\n 2 \n * \n\n 3");
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