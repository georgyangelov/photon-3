// use std::collections::HashMap;
// use crate::frontend::{AST, ASTLiteral, Location};
//
// pub struct HIR {
//     pub node: HIRNode,
//     pub location: Location
// }
//
// pub enum HIRNode {
//     Literal(ASTLiteral),
//     Block(Vec<HIR>)
// }
//
// pub enum Type {
//     Any,
//     Int,
//     Bool,
//     Float,
//     String,
//     Struct { size: usize, fields: () },
//     Interface { size: usize, methods: () }
// }
//
// struct CompilerModule {
//     functions: HashMap<FunctionRef, mir::Function>,
//     types: HashMap<String, mir::Type>,
//     globals: HashMap<String, mir::Global>
// }
//
// enum CompileError {}
//
// pub struct MIR {
//     pub node: MIRNode,
//
//     // This is the important bit of MIR + the references
//     pub typ: mir::Type,
//     pub location: Location
// }
//
// pub enum MIRNode {
//     GlobalRef(i32),
//     Block(Vec<MIR>),
//     Call(FunctionRef, Vec<MIR>),
//     Let(i32, Box<MIR>),
//     LocalRef(i32),
//
//     // Can be done using a function?
//     GetStructFieldValue(Box<MIR>, i32)
// }
//
// pub struct FunctionRef {}
// pub type StructInstanceRef = *mut u8;
// pub type InterfaceInstanceRef = *mut InterfaceInstance;
//
// pub enum Any {
//     Int(i64),
//     Bool(bool),
//     Float(f64),
//     String(Box<str>),
//     StructInstanceRef(StructInstanceRef),
//     InterfaceInstanceRef(InterfaceInstanceRef)
// }
//
// pub struct InterfaceInstance {
//     method_table: Vec<FunctionRef>,
//     value: StructInstanceRef
// }
//
// fn compile_evaluate(compiler: &mut CompilerModule, ast: AST) -> Result<MIR, CompileError> {
//     todo!()
// }
//
// struct WASM {}
// fn compile_mir(compiler: &CompilerModule, mir: MIR) -> Result<WASM, CompileError> {
//     todo!()
// }