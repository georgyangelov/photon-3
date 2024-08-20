extern crate core;

use crate::llvm_test::llvm_test;

mod ast;
mod tests;
mod mir;
mod llvm_test;
mod llvm_test_2;
mod old_lir;
mod types;
mod old_compiler;
mod ir;
mod compiler;
mod ref_registry;
mod vec_map;
mod lir;

fn main() {
    unsafe { llvm_test() }
}
