use std::io;
use std::io::Write;
use lib::Any;
use crate::backend::llvm::LLVMJITCompiler;
use crate::compiler;
use crate::compiler::{mir, ModuleCompiler};
use crate::frontend::{AST, Lexer, ParseError, Parser};

#[test]
fn test_literals() {
    assert_eq!(run("42"), Any::int(42))
}

#[test]
fn test_locals() {
    assert_eq!(run("
        val a = 42
        val b = 11

        a
    "), Any::int(42));
}

#[test]
fn test_add() {
    assert_eq!(run("
        val a = 41
        val b = 1

        a + b
    "), Any::int(42));
}

#[test]
fn test_fns() {
    assert_eq!(run("
        val add = (a, b) a + b

        add(1, 41)
    "), Any::int(42));
}

#[test]
fn test_local_captures() {
    assert_eq!(run("
        val a = 41
        val add = (b) a + b

        add(1)
    "), Any::int(42));
}

#[test]
fn test_param_captures() {
    assert_eq!(run("
        val add = (a) (b) a + b

        add(1)(41)
    "), Any::int(42));
}

#[test]
fn test_capture_captures() {
    assert_eq!(run("
        val a = 41
        val add = () {
            (b) a + b
        }

        add()(1)
    "), Any::int(42));
}

#[test]
fn test_comptime_vals() {
    assert_eq!(run("
        @val a = 41

        a + 1
    "), Any::int(42))
}

fn run(code: &str) -> Any {
    let ast = parse(code).expect("Could not parse");
    let module = ModuleCompiler::compile_module(ast).expect("Could not compile");

    let mut comptime_jit = LLVMJITCompiler::new(&module, true);
    let mut runtime_jit = LLVMJITCompiler::new(&module, false);

    let (comptime_result, comptime_exports) = run_comptime(&mut comptime_jit);
    let runtime_result = run_runtime(&mut runtime_jit, comptime_exports);

    runtime_result
}

fn run_comptime<'a>(jit: &'a mut LLVMJITCompiler) -> (Any, Vec<&'a Any>) {
    let main_fn = jit.compile();

    let result = unsafe { main_fn() };
    let exports = jit.comptime_exports();

    println!("Comptime exports: {:?}", exports);

    (result, exports)
}

fn run_runtime(jit: &mut LLVMJITCompiler, comptime_exports: Vec<&Any>) -> Any {

    jit.set_comptime_exports(comptime_exports);

    let main_fn = jit.compile();

    let result = unsafe { main_fn() };

    result
}

fn parse(code: &str) -> Result<AST, ParseError> {
    let lexer = Lexer::new("<test>", code.chars());
    let mut parser = Parser::new(lexer);

    parser.read_all_as_block()
}