mod ir;
mod value;
mod r#type;
mod compiler;
mod globals;
pub(crate) mod lexical_scope;

pub use ir::*;
pub use value::*;
pub use r#type::*;
pub use compiler::*;
pub use globals::*;