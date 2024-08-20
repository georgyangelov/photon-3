use std::time::Instant;
use crate::{ast, old_lir, mir};
use crate::old_compiler::llvm;
use crate::old_lir::Globals;

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
        val add = (b: Int) a + b

        add(1)
    "), 42);
}

#[test]
fn test_param_captures() {
    assert_eq!(run::<i64>("
        val add = (a: Int) (b: Int) a + b

        add(1)(41)
    "), 42);
}

#[test]
fn test_non_capture_nested_lambdas() {
    assert_eq!(run::<i64>("
        val get_fn = () {
            (a: Int) a + 1
        }

        val fn = get_fn()

        fn(41)
    "), 42);
}

#[test]
fn test_capture_captures() {
    assert_eq!(run::<i64>("
        val a = 41
        val add = () {
            (b: Int) a + b
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

#[test]
fn test_comptime_fn_calls() {
    assert_eq!(run::<i64>("
        @val add_one = (a) a + 1

        @add_one(41)
    "), 42)
}

#[test]
fn test_comptime_captures() {
    assert_eq!(run::<i64>("
        @val add = (a) (b) a + b

        @add(41)(1)
    "), 42)
}

#[test]
fn test_using_types_in_comptime() {
    assert_eq!(run::<i64>("
        @val add = (a: Int, b: Int): Int a + b

        @add(1, 41)
    "), 42)
}

#[test]
fn test_comptime_dynamic_closures() {
    assert_eq!(run::<i64>("
        @(
            val add = (a, b) a + b
            val fn = (): Any { add }

            fn()(1, 41)
        )
    "), 42)
}

#[test]
fn test_using_comptime_vars_for_types_in_block() {
    assert_eq!(run::<i64>("
        @(
            val myInt = Int
            val add = (a: myInt, b: myInt): myInt a + b

            add(1, 41)
        )
    "), 42)
}

// #[test]
// fn test_struct_defs_using_methods() {
//     assert_eq!(run::<i64>("
//         @val Person = Compiler.define_struct \"Person\", {
//           define \"age\", Int
//           define \"height\", Float
//         }
//
//         val person = Person.new(42, 42.0)
//         person.age
//     "), 42)
// }
//
// #[test]
// fn test_struct_defs() {
//     assert_eq!(run::<i64>("
//         struct Person {
//           def age: Int
//           def height: Float
//         }
//
//         val person = Person.new(42, 42.0)
//         person.age
//     "), 42)
// }

fn run<T>(code: &str) -> T {
    let globals = Globals::new();

    let instant = Instant::now();
    let ast = parse(code).expect("Could not parse");
    println!("Parse time: {}ms", instant.elapsed().as_micros() as f64 / 1000f64);

    let instant = Instant::now();
    let mir_module = mir::Compiler::compile_module(ast, &globals).expect("Could not compile");
    println!("MIR compile time: {}ms", instant.elapsed().as_micros() as f64 / 1000f64);

    // println!("Comptime MIR: {:?}", mir_module.comptime_main);

    let instant = Instant::now();
    let comptime_state = old_lir::CompileTimeInterpreter::new(&globals, &mir_module).eval();
    println!("Comptime interpret time: {}ms", instant.elapsed().as_micros() as f64 / 1000f64);

    let instant = Instant::now();
    let lir_module = old_lir::Compiler::compile(&globals, &mir_module, comptime_state);
    println!("LIR compile time: {}ms", instant.elapsed().as_micros() as f64 / 1000f64);

    let instant = Instant::now();
    let mut jit_compiler = llvm::JITCompiler::new(&lir_module);

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