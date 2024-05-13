use std::ffi::CString;

pub struct SymbolNameCounter {
    next_id: u64
}

impl SymbolNameCounter {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    pub fn next(&mut self, prefix: &str) -> CString {
        let str = CString::new(format!("{}.{}", prefix, self.next_id)).unwrap();

        self.next_id += 1;

        str
    }
}
