use crate::{ast, lir, mir};
use crate::compiler::llvm;
use crate::lir::Value;

#[test]
fn test_literals() {
    assert_eq!(run("42"), Value::Int(42))
}

#[test]
fn test_locals() {
    assert_eq!(run("
        val a = 42
        val b = 11

        a
    "), Value::Int(42));
}

#[test]
fn test_add() {
    assert_eq!(run("
        val a = 41
        val b = 1

        a + b
    "), Value::Int(42));
}

#[test]
fn test_fns() {
    assert_eq!(run("
        val add = (a, b) a + b

        add(1, 41)
    "), Value::Int(42));
}

#[test]
fn test_local_captures() {
    assert_eq!(run("
        val a = 41
        val add = (b) a + b

        add(1)
    "), Value::Int(42));
}

#[test]
fn test_param_captures() {
    assert_eq!(run("
        val add = (a) (b) a + b

        add(1)(41)
    "), Value::Int(42));
}

#[test]
fn test_capture_captures() {
    assert_eq!(run("
        val a = 41
        val add = () {
            (b) a + b
        }

        add()(1)
    "), Value::Int(42));
}

#[test]
fn test_comptime_vals() {
    assert_eq!(run("
        @val a = 41

        a + 1
    "), Value::Int(42))
}

#[test]
fn test_comptime_exprs() {
    assert_eq!(run("
        1 + @(1 + 40)
    "), Value::Int(42))
}

#[test]
fn test_using_comptime_vals_in_comptime_exprs() {
    assert_eq!(run("
        @val a = 40

        1 + @(1 + a)
    "), Value::Int(42))
}

fn run(code: &str) -> Value {
    let ast = parse(code).expect("Could not parse");
    let mir_module = mir::Compiler::compile_module(ast).expect("Could not compile");

    let comptime_result = lir::Interpreter::eval_comptime(&mir_module);
    let lir_module = lir::Compiler::compile(&mir_module, comptime_result.exports);
    let mut jit_compiler = llvm::JITCompiler::new(&lir_module);

    let main_fn = jit_compiler.compile();

    let result = unsafe { main_fn() };

    result
}

fn parse(code: &str) -> Result<ast::AST, ast::ParseError> {
    let lexer = ast::Lexer::new("<test>", code.chars());
    let mut parser = ast::Parser::new(lexer);

    parser.read_all_as_block()
}