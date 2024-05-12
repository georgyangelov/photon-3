// use std::fs::File;
// use std::io::Write;
// // use wasmtime::{Engine, Linker, Module, Store};
// use lib::{ValueT, ValueV};
// // use crate::backend::wasm::WasmCompiler;
// use crate::compiler::ModuleCompiler;
// use crate::frontend::{AST, Lexer, ParseError, Parser};
//
// #[test]
// fn test_literals() {
//     assert_eq!(run("42"), ValueT::i64(42))
// }
//
// #[test]
// fn test_locals() {
//     assert_eq!(run("
//         val a = 42
//         val b = 11
//
//         a
//     "), ValueT::i64(42));
// }
//
// // #[test]
// // fn test_intrinsic_calls() {
// //     assert_eq!(run("
// //         1 + 41
// //     "), ValueT::i64(42));
// // }
//
// fn run(code: &str) -> (ValueT, ValueV) {
//     let ast = parse(code).expect("Could not parse");
//     let module = ModuleCompiler::compile_module(ast).expect("Could not compile");
//
//     let mut wasm_compiler = WasmCompiler::new(&module);
//     let wasm_bytecode = wasm_compiler.compile();
//
//     // {
//     //     let mut file = File::create("target/test.wasm").unwrap();
//     //     file.write_all(wasm_bytecode.as_slice()).unwrap();
//     // }
//
//     run_wasm(&wasm_bytecode)
// }
//
// fn parse(code: &str) -> Result<AST, ParseError> {
//     let lexer = Lexer::new("<test>", code.chars());
//     let mut parser = Parser::new(lexer);
//
//     parser.read_all_as_block()
// }
//
// fn run_wasm(bytecode: &[u8]) -> (ValueT, ValueV) {
//     let engine = Engine::default();
//     let mut store = Store::new(&engine, ());
//
//     let built_module = Module::new(&engine, bytecode).expect("Could not load generated wasm binary");
//
//     let mut linker = Linker::new(&engine);
//     let built_module_instance = linker.instantiate(&mut store, &built_module).expect("Could not instantiate built module instance");
//
//     let main_fn = built_module_instance.get_typed_func::<(), (i32, i64)>(&mut store, "main").unwrap();
//
//     let (t, v) = main_fn.call(&mut store, ()).unwrap();
//
//     unsafe { ValueT::from_raw(t, v) }
// }