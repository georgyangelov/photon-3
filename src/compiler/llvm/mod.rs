mod compiler;
mod symbol_name_counter;
mod function_compiler;
mod compiler_module_context;

pub use compiler::*;

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0").as_ptr() as *const i8
    );
}

pub(crate) use c_str;