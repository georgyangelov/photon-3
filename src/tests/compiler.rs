use crate::compiler::ModuleCompiler;
use crate::frontend::{AST, Lexer, ParseError, Parser};
use crate::interpreter::{Interpreter, Value};

macro_rules! expect {
    ($a:expr, $b:pat) => {
        assert_match!(run($a), $b);
    };
}

macro_rules! assert_match {
    ($a:expr, $b:pat) => {
        {
            let actual = $a;
            assert!(matches!(actual, $b), "Expected {:?} to match {}", actual, stringify!($b))
        }
    };
}

#[test]
fn test_simple_expressions() {
    expect!("42", Value::I64(42));
    expect!("42.3", Value::F64(42.3));
    expect!("true", Value::I8(1));
    expect!("false", Value::I8(0));
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
    expect!("41 + 1", Value::I64(42))
}

#[test]
fn test_defining_photon_functions() {
    expect!("
        val fn = (a, b) a + b

        fn(41, 1)
    ", Value::I64(42))
}

// fn expect(code: &str, expected: Value) {
//     expect!(run(code), expected)
// }

fn run(code: &str) -> Value {
    let ast = parse(code).expect("Could not parse");
    let module = ModuleCompiler::compile_module(ast).expect("Could not compile");

    let mut interpreter = Interpreter::new();

    interpreter.eval_module(module)
}

fn parse(code: &str) -> Result<AST, ParseError> {
    let lexer = Lexer::new("<test>", code.chars());
    let mut parser = Parser::new(lexer);

    parser.read_all_as_block()
}