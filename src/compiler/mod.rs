mod compiler;
mod function_builder;
mod jit_compiler;
mod symbol_name_counter;

pub use jit_compiler::*;

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0").as_ptr() as *const i8
    );
}

pub(crate) use c_str;