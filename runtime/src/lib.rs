// pub mod tests;

// extern crate wee_alloc;

use lib::{ValueT, ValueV};

// Use `wee_alloc` as the global allocator.
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[no_mangle]
extern fn call_fn(
    name_t: ValueT, name_v: ValueV,
    // args_t: ValueT, args_v: ValueV
) -> (ValueT, ValueV) {
    let name_str_array = name_t.unwrap_array(name_v);
    // let args_array = args_t.assert_array(args_v);

    let name = unsafe {
        let name_slice = core::slice::from_raw_parts(name_str_array.ptr as *const u8, name_str_array.size as usize);

        // std::str::from_utf8_unchecked(name_slice)
        core::str::from_utf8_unchecked(name_slice)
    };

    if name == "++" {
        ValueT::i64(32)
    } else {
        ValueT::i64(1)
    }
}