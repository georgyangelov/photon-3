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

#[test]
fn test_names() {
    assert_parse("test \n test_two \n test3", "test test_two test3");
    assert_parse("asdf$test \n $test_two", "asdf$test $test_two");
}

#[test]
fn test_method_calls() {
    assert_parse("method", "method");
    assert_parse("method()", "(method self)");
    assert_parse("target.method", "(method target)");
    assert_parse("target.method()", "(method target)");

    assert_parse("a()", "(a self)");
}

#[test]
fn test_method_calls_with_arguments() {
    assert_parse("method(a)", "(method self a)");
    assert_parse("method a", "(method self a)");

    assert_parse("method(a, b, c)", "(method self a b c)");
    assert_parse("method a, b, c", "(method self a b c)");

    assert_parse("method a, \n\n b,\n c", "(method self a b c)");

    assert_parse("one.method a, \n\n b,\n c\n two.d", "(method one a b c) (d two)");

    assert_parse("target.method(a)", "(method target a)");
    assert_parse("target.method a", "(method target a)");

    assert_parse("target.method(a, b, c)", "(method target a b c)");
    assert_parse("target.method a, b, c", "(method target a b c)");
}

#[test]
fn test_methods_with_operator_names() {
    assert_parse("true.!()", "(! true)");
    assert_parse("1.+(42)", "(+ 1 42)");
    assert_parse("1.*(42)", "(* 1 42)");
    assert_parse("1.==(42)", "(== 1 42)");
    assert_parse("1.+=(42)", "(+= 1 42)");
    assert_parse("true.and(false)", "(and true false)");
}

#[test]
fn test_method_chaining() {
    assert_parse("a.b.c", "(c (b a))");
    assert_parse("a.b.c.d e", "(d (c (b a)) e)");
    assert_parse("a.b(1).c", "(c (b a 1))");
    assert_parse("a.b(1).c 2, 3", "(c (b a 1) 2 3)");
    assert_parse("a.b(1).c 2.d, d(3, 4)", "(c (b a 1) (d 2) (d self 3 4))");
}

#[test]
fn test_argument_associativity() {
    assert_parse("one two a, b", "(one self (two self a b))");
    assert_parse("one two(a), b", "(one self (two self a) b)");
    assert_parse("one(two a, b)", "(one self (two self a b))");
    assert_parse("one two(a, b), c, d", "(one self (two self a b) c d)");

    assert_parse("method a, \n\n b,\n c\n d", "(method self a b c) d");
    assert_parse("method(a, \n\n b,\n c\n); d", "(method self a b c) d");
    assert_parse("method a, \n\n b,\n c d e f", "(method self a b (c self (d self (e self f))))");
}

#[test]
fn test_method_call_priority() {
    assert_parse("one a + b", "(one self (+ a b))");
    assert_parse("one(a) + b", "(+ (one self a) b)");

    assert_parse("one a, b + c", "(one self a (+ b c))");
    assert_parse("one(a, b) + c", "(+ (one self a b) c)");
}

#[test]
fn test_fns() {
    assert_parse("{ a\n b\n }", "(fn [] { a b })");
    assert_parse("(a, b) { a\n b\n }", "(fn [(param a) (param b)] { a b })");
    assert_parse("{ a\n }.call 42", "(call (fn [] a) 42)");
    assert_parse("{ a\n }(42)", "(call (fn [] a) 42)");
}

#[test]
fn test_fns_with_a_single_expression() {
    assert_parse("(a, b) a + b", "(fn [(param a) (param b)] (+ a b))");
    assert_parse("((a, b) a + b)()", "(call (fn [(param a) (param b)] (+ a b)))");

    // TODO: These should be errors
    // assert_parse("(a, b) (a + b)", "(fn [(param a) (param b)] { (+ a b) })");
    // assert_parse("((a, b) (a + b))()", "(call (fn [(param a) (param b)] { (+ a b) }))");
    // assert_parse("((a, b) (a) + (b))()", "(call (fn [(param a) (param b)] { (+ a b) }))");
    // assert_parse("((a, b) ((a) + (b)))()", "(call (fn [(param a) (param b)] { (+ a b) }))");

    assert_parse("() a + b", "(fn [] (+ a b))");
    assert_parse("(() a + b)()", "(call (fn [] (+ a b)))");
}

#[test]
fn test_ambiguous_fn_cases() {
    assert_parse("array.forEach (element) { element + 1 }", "(forEach array (fn [(param element)] (+ element 1)))");
    assert_parse("forEach (element) { element + 1 }", "(forEach self (fn [(param element)] (+ element 1)))");

    // TODO: These should be errors
    // assert_parse("array.forEach(element) { element + 1 }", "(forEach array (fn [(param element)] { (+ element 1) }))");
    // assert_parse("forEach (element) { element + 1 }", "(forEach self (fn [(param element)] { (+ element 1) }))");

    assert_parse("array.forEach(element)", "(forEach array element)");
    assert_parse("forEach(element)", "(forEach self element)");

    assert_parse("(a){ a + 41 }(1)", "(call (fn [(param a)] (+ a 41)) 1)");
    assert_parse("((a){ a + 41 })(1)", "(call (fn [(param a)] (+ a 41)) 1)");

    // TODO: This is an error, the argument list must not have whitespace before the open paren
    // assert_parse("(a){ a + 41 } (1)", "(fn [(param a)] { (+ a 41) }) 1");
}

#[test]
fn test_nested_lambda_calls() {
    assert_parse("(a) { (b) { a + b } }(1)(41)", "(call (call (fn [(param a)] (fn [(param b)] (+ a b))) 1) 41)");
}

#[test]
fn test_lambdas_using_only_braces() {
    assert_parse("{ a }", "(fn [] a)");
    assert_parse("val a = 42; { a }", "(let a 42) (fn [] a)");
}

#[test]
fn test_type_annotations_on_values() {
    assert_parse("42: Int", "(type-assert 42 Int)");
    assert_parse("a: Int", "(type-assert a Int)");
    assert_parse("\"hello\": String", "(type-assert \"hello\" String)");
    assert_parse("42: Map(String, List(Int))", "(type-assert 42 (Map self String (List self Int)))");

    // TODO: What about this?
    // assertParse("fn(42): Int", "(type-assert (fn self 42) Int)")
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