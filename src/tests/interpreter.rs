use crate::compiler::MirModuleCompiler;
use crate::frontend::{AST, Lexer, ParseError, Parser};
use crate::backend::interpreter::{Interpreter, Value};

macro_rules! expect {
    ($a:expr, $b:pat) => {
        {
            let (comptime_exports, result) = run($a);

            assert_match!(result, $b);

            (comptime_exports, result)
        }
    };
}

macro_rules! assert_match {
    ($a:expr, $b:pat) => {
        {
            let actual = &$a;
            assert!(matches!(actual, &$b), "Expected {:?} to match {}", actual, stringify!($b))
        }
    };
}

#[test]
fn test_simple_expressions() {
    expect!("42", Value::I64(42));
    expect!("42.3", Value::F64(42.3));
    expect!("true", Value::Bool(true));
    expect!("false", Value::Bool(false));
}

#[test]
fn test_variable_assignments() {
    expect!("
        val a = 42
        val b = a

        b
    ", Value::I64(42));
}

#[test]
fn test_variable_assignments_in_blocks() {
    expect!("
        val a = 42

        (
            val b = a

            b
        )
    ", Value::I64(42));
}

#[test]
fn test_adding_numbers() {
    expect!("41 + 1", Value::I64(42));
}

#[test]
fn test_defining_photon_functions() {
    expect!("
        val fn = (a, b) a + b

        fn(41, 1)
    ", Value::I64(42));
}

#[test]
fn test_closures() {
    expect!("
        val a = 41
        val fn = (b) a + b

        fn(1)
    ", Value::I64(42));
}

#[test]
fn test_closures_2() {
    expect!("
        val add_fn = (a) { (b) a + b }
        val add_one = add_fn(1)

        add_one(41)
    ", Value::I64(42));
}

#[test]
fn test_comptime_vars() {
    let (exports, _) = expect!("
        @val a = 42

        a
    ", Value::I64(42));

    assert_match!(&exports[0], &Value::I64(42));
}

#[test]
fn test_comptime_vars_in_fn_scopes() {
    expect!("
        val a = {
            @val a = 40 + 1

            { a + 1 }
        }

        a()()
    ", Value::I64(42));
}

#[test]
fn test_comptime_vars_in_comptime_fns() {
    expect!("
        @val a = {
            @val a = 42

            a
        }

        a()
    ", Value::I64(42));
}

#[test]
fn test_capture_of_comptime_vars_in_comptime_fns() {
    expect!("
        @val a = 41
        @val fn = {
            a + 1
        }

        fn()
    ", Value::I64(42));
}

#[test]
fn test_comptime_expressions() {
    let (exports, _) = expect!("
        @val a = 39

        1 + @(a + 1) + 1
    ", Value::I64(42));

    assert_match!(&exports[0], &Value::I64(40));
}

#[test]
fn test_ifs() {
    expect!("if true then 42 else 1", Value::I64(42));
    expect!("if false then 42 else 1", Value::I64(1));
}

// fn expect(code: &str, expected: Value) {
//     expect!(run(code), expected)
// }

fn run(code: &str) -> (Vec<Value>, Value) {
    let ast = parse(code).expect("Could not parse");
    let module = MirModuleCompiler::compile_module(ast).expect("Could not compile");

    let mut interpreter = Interpreter::new();

    let comptime_result = interpreter.eval_module_comptime(&module);
    let exports = comptime_result.comptime_exports.clone();

    let result = interpreter.eval_module_runtime(&module, comptime_result.comptime_exports);

    (exports, result)
}

fn parse(code: &str) -> Result<AST, ParseError> {
    let lexer = Lexer::new("<test>", code.chars());
    let mut parser = Parser::new(lexer);

    parser.read_all_as_block()
}