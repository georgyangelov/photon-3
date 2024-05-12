use llvm_sys::prelude::LLVMValueRef;

pub struct RefTable {
    pub table: Vec<LLVMValueRef>
}

impl RefTable {
    pub fn new(capacity: usize) -> Self {
        let mut table = Vec::with_capacity(capacity);

        Self { table }
    }
}