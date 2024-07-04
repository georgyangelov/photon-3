use crate::{ast, lir, mir};
use crate::compiler::llvm;
use crate::lir::{Globals, Value};

#[test]
fn test_literals() {
    assert_eq!(run::<i64>("42"), 42)
}

#[test]
fn test_locals() {
    assert_eq!(run::<i64>("
        val a = 42
        val b = 11

        a
    "), 42);
}

#[test]
fn test_add() {
    assert_eq!(run::<i64>("
        val a = 41
        val b = 1

        a + b
    "), 42);
}

#[test]
fn test_fns() {
    assert_eq!(run::<i64>("
        val add = (a: Int, b: Int) a + b

        add(1, 41)
    "), 42);
}

#[test]
fn test_local_captures() {
    assert_eq!(run::<i64>("
        val a = 41
        val add = (b) a + b

        add(1)
    "), 42);
}

#[test]
fn test_param_captures() {
    assert_eq!(run::<i64>("
        val add = (a) (b) a + b

        add(1)(41)
    "), 42);
}

#[test]
fn test_capture_captures() {
    assert_eq!(run::<i64>("
        val a = 41
        val add = () {
            (b) a + b
        }

        add()(1)
    "), 42);
}

#[test]
fn test_comptime_vals() {
    assert_eq!(run::<i64>("
        @val a = 41

        a + 1
    "), 42)
}

#[test]
fn test_comptime_exprs() {
    assert_eq!(run::<i64>("
        1 + @(1 + 40)
    "), 42)
}

#[test]
fn test_using_comptime_vals_in_comptime_exprs() {
    assert_eq!(run::<i64>("
        @val a = 40

        1 + @(1 + a)
    "), 42)
}

fn run<T>(code: &str) -> T {
    let globals = Globals::new();

    let ast = parse(code).expect("Could not parse");
    let mir_module = mir::Compiler::compile_module(ast, &globals).expect("Could not compile");

    let comptime_state = lir::CompileTimeInterpreter::new(&globals, &mir_module).eval();
    let lir_module = lir::Compiler::compile(&globals, &mir_module, comptime_state);
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