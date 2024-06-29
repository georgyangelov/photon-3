// extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use std::alloc::Layout;
use std::ffi::{CStr};
use lib::{Any, AnyT};

pub extern fn call(name: *const i8, args: *const Any, arg_count: u64) -> Any {
    unsafe {
        let name = CStr::from_ptr(name);
        let args = std::slice::from_raw_parts(args, arg_count as usize);

        println!("name: {:?}", name);
        println!("args: {:?}", args);

        if name == c"+" {
            Any::int(args[0].assert_int() + args[1].assert_int())
        } else if name == c"call" && args[0].typ == AnyT::Closure {
            let this = args[0];

            // Non-trampoline option
            // match arg_count - 1 {
            //     0 => this.fn_0()(),
            //     1 => this.fn_1()(args[1]),
            //     2 => this.fn_2()(args[1], args[2]),
            //     3 => this.fn_3()(args[1], args[2], args[3]),
            //     _ => panic!("Functions with more than 3 parameters are not supported")
            // }

            // Trampoline option
            let (func, closure_struct) = this.trampoline_closure();

            func(args.as_ptr().add(1), closure_struct) // .byte_add(size_of::<usize>())
        } else if name == c"call" && args[0].typ == AnyT::FunctionPtr {
            let this = args[0];
            let func = this.trampoline_fn();

            func(args.as_ptr().add(1))
        } else {
            panic!("Unknown function {}", name.to_str().unwrap())
        }
    }
}

pub extern fn malloc(size: usize) -> *mut u8 {
    let res = unsafe { std::alloc::alloc(Layout::array::<u8>(size).unwrap()) };

    res
}
