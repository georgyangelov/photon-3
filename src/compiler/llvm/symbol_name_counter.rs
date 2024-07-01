use std::ffi::CString;

pub struct SymbolNameCounter {
    next_id: u64
}

impl SymbolNameCounter {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    pub fn next_string(&mut self, prefix: &str) -> String {
        let name = format!("{}.{}", prefix, self.next_id);

        self.next_id += 1;

        name
    }

    pub fn next(&mut self, prefix: &str) -> CString {
        CString::new(self.next_string(prefix)).unwrap()
    }
}
