extern crate core;

use crate::llvm_test::llvm_test;

mod ast;
mod tests;
mod mir;
mod llvm_test;
mod llvm_test_2;
mod lir;
mod types;
mod lib;
mod compiler;

fn main() {
    unsafe { llvm_test() }
}
