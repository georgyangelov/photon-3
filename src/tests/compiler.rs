use std::time::Instant;
use crate::{ast, compiler, ir};
use crate::ir::Globals;

#[test]
fn test_literals() {
    assert_eq!(run::<i64>("42"), 42)
}

fn run<T>(code: &str) -> T {
    let globals = Globals::new();

    let instant = Instant::now();
    let ast = parse(code).expect("Could not parse");
    println!("Parse time: {}ms", instant.elapsed().as_micros() as f64 / 1000f64);

    let instant = Instant::now();
    let module = ir::Builder::build_module(ast, &globals);
    println!("IR compile time: {}ms", instant.elapsed().as_micros() as f64 / 1000f64);

    let instant = Instant::now();
    let module = ir::Interpreter::eval_comptime(&globals, module);
    println!("{:?}", module.main.body);
    println!("Comptime interpret time: {}ms", instant.elapsed().as_micros() as f64 / 1000f64);

    let instant = Instant::now();
    let mut jit_compiler = compiler::JITCompiler::new(&module);

    let main_fn = jit_compiler.compile();
    println!("LLVM compile time: {}ms", instant.elapsed().as_micros() as f64 / 1000f64);

    let instant = Instant::now();
    let result = unsafe { main_fn() };
    println!("Run time: {}ms", instant.elapsed().as_micros() as f64 / 1000f64);

    result
}

fn parse(code: &str) -> Result<ast::AST, ast::ParseError> {
    let lexer = ast::Lexer::new("<test>", code.chars());
    let mut parser = ast::Parser::new(lexer);

    parser.read_all_as_block()
}