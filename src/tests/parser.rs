use crate::ast::{Lexer, ParseError, Parser};

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

// TODO: Remove this, we don't need this
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
fn test_nested_fn_calls() {
    assert_parse("(a) { (b) { a + b } }(1)(41)", "(call (call (fn [(param a)] (fn [(param b)] (+ a b))) 1) 41)");
}

#[test]
fn test_fns_using_only_braces() {
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

#[test]
fn test_requires_parens_on_types_of_parameters() {
    assert_parse_error("(param1: fn arg1, b: Int) 42")
}

#[test]
fn test_patterns_in_parameter_types() {
    assert_parse("(a: val AT) 42", "(fn [(param a (val AT))] 42)");
    assert_parse("(a: val AT, b: AT) 42", "(fn [(param a (val AT)) (param b AT)] 42)");
}

#[test]
fn test_call_patterns_in_parameter_types() {
    assert_parse("(a: Optional(val T)) 42", "(fn [(param a <Optional self (val T)>)] 42)");
    assert_parse("(a: Optional.match(val T)) 42", "(fn [(param a <match Optional (val T)>)] 42)");
    assert_parse("(a: Optional.match(1, val T)) 42", "(fn [(param a <match Optional 1 (val T)>)] 42)");
    assert_parse("(a: Optional.match(1, Optional(val T))) 42", "(fn [(param a <match Optional 1 <Optional self (val T)>>)] 42)");
}

#[test]
fn test_specific_value_patterns_in_parameter_types() {
    assert_parse("(a: Optional(Int)) 42", "(fn [(param a (Optional self Int))] 42)");
    assert_parse("(a: Optional.of(Int)) 42", "(fn [(param a (of Optional Int))] 42)");
    assert_parse("(a: Optional.of(1, Int)) 42", "(fn [(param a (of Optional 1 Int))] 42)");
}

#[test]
fn test_type_annotation_on_fn_return_type() {
    assert_parse("(a: Int): Int { a + 1 }", "(fn [(param a Int)] Int (+ a 1))");
    assert_parse("(a: Int): Int a + 1", "(fn [(param a Int)] Int (+ a 1))");
}

#[test]
fn test_compile_time_parameters() {
    assert_parse("(@a): Int { a + 1 }", "(fn [(@param a)] Int (+ a 1))");
    assert_parse("(@a: Int): Int { a + 1 }", "(fn [(@param a Int)] Int (+ a 1))");
}

#[test]
fn test_type_annotations_on_val() {
    assert_parse("val a: Int = 42; a", "(let a (type-assert 42 Int)) a");
    assert_parse("val a: Stream(Int) = 42; a", "(let a (type-assert 42 (Stream self Int))) a");
}

#[test]
fn test_parens_for_blocks() {
    assert_parse("(a; b)", "{ a b }");
    assert_parse("(a; b) + 1", "(+ { a b } 1)");
    assert_parse("val a = 11; (val a = 42; () { a }) + a", "(let a 11) (+ { (let a 42) (fn [] a) } a)");

    assert_parse_error("()");
}

#[test]
fn test_method_call_precedence() {
    assert_parse("array.map { 42 }.filter (x) x > 0", "(map array (filter (fn [] 42) (fn [(param x)] (> x 0))))");
    assert_parse("array.map { 42 } .filter (x) x > 0", "(filter (map array (fn [] 42)) (fn [(param x)] (> x 0)))");
    assert_parse("array.map({ 42 }).filter (x) x > 0", "(filter (map array (fn [] 42)) (fn [(param x)] (> x 0)))");
    assert_parse("array.map { 42 }\n.filter (x) x > 0", "(filter (map array (fn [] 42)) (fn [(param x)] (> x 0)))");

    assert_parse("array.map 42.filter", "(map array (filter 42))");
    assert_parse("array.map(42 .filter)", "(map array (filter 42))");
    assert_parse("array.map 42 .filter", "(filter (map array 42))");
    assert_parse("array.map(42).filter", "(filter (map array 42))");

    assert_parse("array.map 1 + 2.filter", "(map array (+ 1 (filter 2)))");
    assert_parse("array.map 1 + 2 .filter", "(filter (map array (+ 1 2)))");
    assert_parse("array.map 1.to_s + 2 .filter", "(filter (map array (+ (to_s 1) 2)))");

    assert_parse("array.map array2.map 1 .filter", "(filter (map array (map array2 1)))");

    assert_parse("map 42.filter", "(map self (filter 42))");
    assert_parse("map(42 .filter)", "(map self (filter 42))");
    assert_parse("map 42 .filter", "(filter (map self 42))");
    assert_parse("map(42).filter", "(filter (map self 42))");
}

#[test]
fn test_types_for_function_values() {
    assert_parse("val a: (): Int = () 42; a", "(let a (type-assert (fn [] 42) (fn-type [] Int))) a");
    assert_parse("val a: ((): Int) = () 42; a", "(let a (type-assert (fn [] 42) (fn-type [] Int))) a");
}

#[test]
fn test_function_types_with_argument_types() {
    assert_parse("(a: Int): Int", "(fn-type [(param a Int)] Int)");
    assert_parse("(a: Int, b: String): Int", "(fn-type [(param a Int) (param b String)] Int)");
}

#[test]
fn test_generic_fns_with_patterns_in_fn_types() {
    assert_parse("val fn = (a: (n: val T): T) a(42); fn((a) a)", "(let fn (fn [(param a (fn-type [(param n (val T))] T))] (a self 42))) (fn self (fn [(param a)] a))")
}

#[test]
fn test_compile_time_expressions() {
    assert_parse("@42", "@42");
    assert_parse("@name", "@name");
    assert_parse("@(1 + 1)", "@(+ 1 1)");
    assert_parse("@array.map 42", "@(map array 42)");
    assert_parse("val a = @{ 42 }", "(let a @(fn [] 42))");
    assert_parse("@1 + 1", "(+ @1 1)");
    assert_parse("@method(1 + 1)", "@(method self (+ 1 1))");
    assert_parse("@method 1 + 1", "@(method self (+ 1 1))");
    assert_parse("@method\n1 + 1", "@method (+ 1 1)");
    assert_parse("@method.call 1 + 1", "@(call method (+ 1 1))");
}

#[test]
fn test_compile_time_vals() {
    assert_parse("@val a = 42", "(@let a 42)");
    assert_parse("@rec val a = 42", "(@let-rec a 42)");
}

#[test]
fn test_nested_fns() {
    assert_parse("(a) (b) a + b", "(fn [(param a)] (fn [(param b)] (+ a b)))")
}

#[test]
fn test_ifs() {
    assert_parse("if a { b }", "(if a b)");
    assert_parse("if a then b", "(if a b)");
    assert_parse("if a then b else c", "(if a b c)");
    assert_parse("if a { b } else { c }", "(if a b c)");
    assert_parse("if a { b } else if c { d }", "(if a b (if c d))");
    assert_parse("if a { b } else if c { d } else { e }", "(if a b (if c d e))");
    assert_parse("if a then b else if c then d else e", "(if a b (if c d e))");
}

fn assert_parse(code: &str, expected: &str) {
    let result = parse(code).expect(format!("Could not parse code {}", code).as_str());

    assert_eq!(result, expected)
}

fn assert_parse_error(code: &str) {
    let result = parse(code);

    assert!(matches!(result, Err(_)))
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