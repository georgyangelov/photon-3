use std::ffi::CString;

pub struct AnonCounter {
    next: i32
}

impl AnonCounter {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn next_str(&mut self) -> CString {
        let str = CString::new(format!("anon.{}", self.next)).unwrap();

        self.next += 1;

        str
    }
}