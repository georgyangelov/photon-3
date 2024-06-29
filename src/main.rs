extern crate core;

// use std::ffi::CString;
// use std::mem::size_of;
// use std::os::raw::c_char;
// use std::ptr;
// use std::ptr::slice_from_raw_parts_mut;
// use binaryen_sys::*;
// use wasmtime::{Engine, Instance, Linker, Memory, Module, Store};
// use crate::llvm_test_2::llvm_test_2;
use crate::llvm_test::llvm_test;
// use runtime::Position;

mod ast;
mod tests;
mod mir;
mod compiler;
mod llvm_test;
mod llvm_test_2;
mod lir;

fn main() {
    unsafe { llvm_test() }
}

// fn binaryen_test() {
//     let engine = Engine::default();
//
//     let runtime_module = Module::from_file(&engine, "target/wasm32-unknown-unknown/release/runtime.wasm")
//         .expect("Could not load wasm file into module");
//
//     let mut store = Store::new(&engine, ());
//
//     // let imports = [];
//
//     // let instance = Instance::new(&mut store, &module, &imports)
//     //     .expect("Could not create instance of the module");
//
//     // let function =
//     //     instance.get_typed_func::<(i64, i64), i64>(&mut store, "add")
//     //         .expect("Could not get typed function");
//     //
//     // let result = function.call(&mut store, (1, 41))
//     //     .expect("Could not call function");
//     //
//     // println!("Result: {}", result);
//
//     // let new_struct_fn = instance.get_typed_func::<(), i32>(&mut store, "new_struct")
//     //     .expect("Could not get the new_struct function");
//     //
//     // let drop_struct_fn = instance.get_typed_func::<(i32), ()>(&mut store, "drop_struct")
//     //     .expect("Could not get the drop_struct function");
//     //
//     // let struct_ptr = new_struct_fn.call(&mut store, ())
//     //     .expect("Could not call new_struct");
//     // {
//     //     let memory = instance.get_memory(&mut store, "memory")
//     //         .expect("Could not get memory");
//     //
//     //     unsafe {
//     //         let ptr = memory.data_ptr(&mut store).byte_add(struct_ptr as usize);
//     //         let ptr2 = ptr as *mut Position;
//     //
//     //         println!("Position({}, {})", (*ptr2).x, (*ptr2).y)
//     //     }
//     // }
//     //
//     // drop_struct_fn.call(&mut store, struct_ptr).expect("Could not call drop_struct");
//
//     // let mut memory = instance.get_memory(&mut store, "memory")
//     //         .expect("Could not get memory");
//     //
//     // let new_buffer_fn = instance.get_typed_func::<(i32), i32>(&mut store, "new_buffer").unwrap();
//     // let buffer_read_ptr_fn = instance.get_typed_func::<(i32), i32>(&mut store, "buffer_read_ptr").unwrap();
//     // let buffer_add_two_numbers_fn = instance.get_typed_func::<(i32), i64>(&mut store, "buffer_add_two_numbers").unwrap();
//     // let drop_buffer_fn = instance.get_typed_func::<(i32), ()>(&mut store, "drop_buffer").unwrap();
//     //
//     // let buffer_size = 2;
//     // let buffer_handle = new_buffer_fn.call(&mut store, buffer_size).unwrap();
//     // let buffer_data_ptr = buffer_read_ptr_fn.call(&mut store, buffer_handle).unwrap();
//     //
//     // unsafe {
//     //     let ptr = memory_pointer(&mut memory, &mut store, buffer_data_ptr) as *mut i64;
//     //
//     //     let slice = slice_from_raw_parts_mut(ptr, buffer_size as usize);
//     //     (*slice)[0] = 1;
//     //     (*slice)[1] = 41;
//     //
//     //     // ptr.write(1_u8);
//     //     // ptr.byte_add(8).write(41_u8);
//     // }
//     //
//     // // TODO: Is the `buffer_handle` still a valid pointer here? Since we called a function and that may have
//     // //       changed memory. :think:
//     // let add_result = buffer_add_two_numbers_fn.call(&mut store, buffer_handle).unwrap();
//     //
//     // println!("Add result: {}", add_result);
//     //
//     // drop_buffer_fn.call(&mut store, buffer_handle).unwrap();
//
//     let wasm_binary = build_wasm_module();
//
//     let built_module = Module::new(&engine, wasm_binary).expect("Could not load generated wasm binary");
//
//     let mut linker = Linker::new(&engine);
//
//     let runtime_instance = linker.instantiate(&mut store, &runtime_module).expect("Could not instantiate runtime module");
//     linker.instance(&mut store, "runtime", runtime_instance).expect("Could not name instance of runtime module");
//
//     let built_module_instance = linker.instantiate(&mut store, &built_module).expect("Could not instantiate built module instance");
//
//     // let built_module_instance = Instance::new(&mut store, &built_module, &imports)
//     //     .expect("Could not create instance of the module");
//
//     let adder_fn = built_module_instance.get_typed_func::<(i64, i64), i64>(&mut store, "adder").unwrap();
//
//     let result = adder_fn.call(&mut store, (1, 41)).unwrap();
//
//     println!("Add result: {}", result);
// }

// unsafe fn memory_pointer(memory: &mut Memory, store: &mut Store<()>, offset: i32) -> *mut u8 {
//     memory.data_ptr(store).byte_add(offset as u32 as usize)
// }

// fn build_wasm_module() -> Vec<u8> {
//     let mut output = Vec::<u8>::with_capacity(1 * 1024 * 1024);
//
//     unsafe {
//         let module = BinaryenModuleCreate();
//
//         BinaryenModuleSetFeatures(module, BinaryenFeatureMultivalue());
//
//         let add_internal_name = CString::new("add_imported").unwrap();
//
//         // Import add function from the runtime module
//         {
//             let runtime_module_name = CString::new("runtime").unwrap();
//             let add_external_name = CString::new("add").unwrap();
//
//             let mut params = [BinaryenTypeInt64(), BinaryenTypeInt64()];
//             let params = BinaryenTypeCreate(params.as_mut_ptr(), params.len() as u32);
//             let results = BinaryenTypeInt64();
//
//             BinaryenAddFunctionImport(
//                 module,
//                 add_internal_name.as_ptr(),
//                 runtime_module_name.as_ptr(),
//                 add_external_name.as_ptr(),
//                 params,
//                 results
//             );
//         }
//
//         let func_name = CString::new("adder").unwrap();
//
//         // Define local adder function
//         // {
//         //     let mut params = [BinaryenTypeInt64(), BinaryenTypeInt64()];
//         //     let params = BinaryenTypeCreate(params.as_mut_ptr(), params.len() as u32);
//         //     let results = BinaryenTypeInt64();
//         //
//         //     let x = BinaryenLocalGet(module, 0, BinaryenTypeInt64());
//         //     let y = BinaryenLocalGet(module, 1, BinaryenTypeInt64());
//         //
//         //     // let add = BinaryenBinary(module, BinaryenAddInt64(), x, y);
//         //
//         //     let mut add_call_operands = [x, y];
//         //     let call = BinaryenCall(
//         //         module,
//         //         add_internal_name.as_ptr(),
//         //         add_call_operands.as_mut_ptr(),
//         //         add_call_operands.len() as u32,
//         //         results
//         //     );
//         //
//         //     let _ = BinaryenAddFunction(
//         //         module,
//         //         func_name.as_ptr(),
//         //         params,
//         //         results,
//         //         ptr::null_mut(),
//         //         0,
//         //         call
//         //     );
//         // }
//
//         // Define local nop function to test multivalue
//         {
//             let mut params = [BinaryenTypeInt64(), BinaryenTypeInt64()];
//             let params = BinaryenTypeCreate(params.as_mut_ptr(), params.len() as u32);
//
//             let mut result_types = [BinaryenTypeInt64(), BinaryenTypeInt64()];
//             let results = BinaryenTypeCreate(result_types.as_mut_ptr(), result_types.len() as u32);
//
//             let x = BinaryenLocalGet(module, 0, BinaryenTypeInt64());
//             let y = BinaryenLocalGet(module, 1, BinaryenTypeInt64());
//
//             let mut tuple_args = [x, y];
//             let body = BinaryenTupleMake(module, tuple_args.as_mut_ptr(), tuple_args.len() as u32);
//
//             let _ = BinaryenAddFunction(
//                 module,
//                 func_name.as_ptr(),
//                 params,
//                 results,
//                 ptr::null_mut(),
//                 0,
//                 body
//             );
//         }
//
//         let _ = BinaryenAddFunctionExport(module, func_name.as_ptr(), func_name.as_ptr());
//
//         let is_valid = BinaryenModuleValidate(module);
//         println!("Module valid: {}", is_valid);
//
//         BinaryenModulePrint(module);
//         BinaryenModuleOptimize(module);
//         BinaryenModulePrint(module);
//
//         let size = BinaryenModuleWrite(module, output.as_mut_ptr() as *mut c_char, output.capacity());
//         if size == output.capacity() {
//             panic!("Not enough space in the buffer to save wasm bytecode");
//         }
//
//         output.set_len(size);
//
//         BinaryenModuleDispose(module);
//     }
//
//     output
// }
