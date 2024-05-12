use lib::Value;
use crate::backend::llvm::LLVMJITCompiler;
use crate::compiler::ModuleCompiler;
use crate::frontend::{AST, Lexer, ParseError, Parser};

#[test]
fn test_literals() {
    assert_eq!(run("42"), Value::int(42))
}

#[test]
fn test_locals() {
    assert_eq!(run("
        val a = 42
        val b = 11

        a
    "), Value::int(42));
}

fn run(code: &str) -> Value {
    let ast = parse(code).expect("Could not parse");
    let module = ModuleCompiler::compile_module(ast).expect("Could not compile");

    let mut jit = LLVMJITCompiler::new(&module);
    let main_fn = jit.compile();

    // let mut result = Value::none();

    // unsafe { (*main_fn)(&mut result) };
    let result = unsafe { main_fn() };

    result
}

fn parse(code: &str) -> Result<AST, ParseError> {
    let lexer = Lexer::new("<test>", code.chars());
    let mut parser = Parser::new(lexer);

    parser.read_all_as_block()
}