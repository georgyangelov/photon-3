use std::ffi::CString;

pub struct AnonCounter {
    next: i32
}

impl AnonCounter {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn next_anon(&mut self) -> CString {
        self.next_str("anon")
    }

    pub fn next_str(&mut self, prefix: &str) -> CString {
        let str = CString::new(format!("{}.{}", prefix, self.next)).unwrap();

        self.next += 1;

        str
    }
}