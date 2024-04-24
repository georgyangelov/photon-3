extern crate wee_alloc;

use std::alloc::Layout;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub extern fn add(a: i64, b: i64) -> i64 {
    a + b + 1
}

#[repr(C)]
pub struct Position {
    pub x: i64,
    pub y: i64
}

#[no_mangle]
pub extern fn new_struct() -> Box<Position> {
    Box::new(Position { x: 1, y: 2 })
}

#[no_mangle]
pub extern fn drop_struct(_: Box<Position>) {
}

#[no_mangle]
pub unsafe extern fn allocates(value: i64) -> *mut u8 {
    // Box::new(Test { one: value })

    let layout = Layout::new::<[u16;2]>();
    let ptr = std::alloc::alloc(layout);

    ptr
}

#[no_mangle]
pub extern fn new_buffer(size: usize) -> Box<Vec<i64>> {
    let vector = vec![0; size];

    Box::new(vector)
}

#[no_mangle]
pub extern fn buffer_read_ptr(buffer: &mut Vec<i64>) -> *mut u8 {
    buffer.as_mut_ptr() as *mut u8
}

#[no_mangle]
pub extern fn buffer_add_two_numbers(buffer: &Vec<i64>) -> i64 {
    buffer[0] + buffer[1]
}

#[no_mangle]
pub extern fn drop_buffer(buffer: Box<Vec<u8>>) {
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
