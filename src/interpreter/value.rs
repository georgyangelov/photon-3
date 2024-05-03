#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    None,

    I8(i8),
    // I32(i32),
    I64(i64),
    F64(f64),

    // String(String),

    // Struct()
}

impl Value {
    pub fn expect_i64(self) -> i64 {
        match self {
            Value::I64(value) => value,
            _ => todo!("Error handling")
        }
    }
}