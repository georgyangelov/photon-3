extern crate wee_alloc;

use std::alloc::Layout;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub extern fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[no_mangle]
pub unsafe extern fn allocates(value: i64) -> *mut u8 {
    // Box::new(Test { one: value })

    let layout = Layout::new::<[u16;2]>();
    let ptr = std::alloc::alloc(layout);

    ptr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
