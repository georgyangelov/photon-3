// use std::ffi::CString;
// use std::ptr;
// use llvm_sys::core::{LLVMAddFunction, LLVMAppendBasicBlock, LLVMBuildAdd, LLVMBuildRet, LLVMCreateBuilder, LLVMDisposeBuilder, LLVMFunctionType, LLVMGetParam, LLVMInt32Type, LLVMModuleCreateWithNameInContext, LLVMPositionBuilderAtEnd};
// use llvm_sys::error::{LLVMErrorRef, LLVMGetErrorMessage};
// use llvm_sys::orc2::{LLVMOrcCreateNewThreadSafeContext, LLVMOrcCreateNewThreadSafeModule, LLVMOrcDisposeThreadSafeContext, LLVMOrcJITTargetAddress, LLVMOrcThreadSafeContextGetContext, LLVMOrcThreadSafeModuleRef};
// use llvm_sys::orc2::lljit::{LLVMOrcCreateLLJIT, LLVMOrcLLJITAddLLVMIRModule, LLVMOrcLLJITGetMainJITDylib, LLVMOrcLLJITLookup, LLVMOrcLLJITRef};
// use llvm_sys::support::LLVMParseCommandLineOptions;
// use llvm_sys::target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget};
// use crate::main;
//
// macro_rules! c_str {
//     ($s:expr) => (
//         concat!($s, "\0").as_ptr() as *const i8
//     );
// }
//
// unsafe fn create_demo_module() -> LLVMOrcThreadSafeModuleRef {
//     let tsc = LLVMOrcCreateNewThreadSafeContext();
//     let ctx = LLVMOrcThreadSafeContextGetContext(tsc);
//     let m = LLVMModuleCreateWithNameInContext(c_str!("demo"), ctx);
//
//     // Context
//     let mut param_types = [LLVMInt32Type(), LLVMInt32Type()];
//
//     // Context
//     let sum_function_type = LLVMFunctionType(LLVMInt32Type(), param_types.as_mut_ptr(), 2, 0);
//     let sum_function = LLVMAddFunction(m, c_str!("sum"), sum_function_type);
//
//     // Context
//     let entry_bb = LLVMAppendBasicBlock(sum_function, c_str!("entry"));
//
//     // Context
//     let builder = LLVMCreateBuilder();
//     LLVMPositionBuilderAtEnd(builder, entry_bb);
//
//     let sum_arg_0 = LLVMGetParam(sum_function, 0);
//     let sum_arg_1 = LLVMGetParam(sum_function, 1);
//     let result = LLVMBuildAdd(builder, sum_arg_0, sum_arg_1, c_str!("result"));
//
//     LLVMBuildRet(builder, result);
//     LLVMDisposeBuilder(builder);
//
//     let tsm = LLVMOrcCreateNewThreadSafeModule(m, tsc);
//
//     LLVMOrcDisposeThreadSafeContext(tsc);
//
//     tsm
// }
//
// pub unsafe fn llvm_test_2() {
//     // LLVMParseCommandLineOptions(1, [c_str!("asdf")].as_mut_ptr(), c_str!(""));
//
//     LLVM_InitializeNativeTarget();
//     LLVM_InitializeNativeAsmPrinter();
//
//     let mut j: LLVMOrcLLJITRef = ptr::null_mut();
//     let err = LLVMOrcCreateLLJIT(&mut j, ptr::null_mut());
//     if !err.is_null() {
//         panic!("Error: {}", CString::from_raw(LLVMGetErrorMessage(err)).into_string().unwrap())
//     }
//
//     let tsm = create_demo_module();
//
//     let main_jd = LLVMOrcLLJITGetMainJITDylib(j);
//
//     let err = LLVMOrcLLJITAddLLVMIRModule(j, main_jd, tsm);
//     if !err.is_null() {
//         panic!("Error: {}", CString::from_raw(LLVMGetErrorMessage(err)).into_string().unwrap())
//     }
//
//     let mut sum_addr: LLVMOrcJITTargetAddress = 0;
//     let err = LLVMOrcLLJITLookup(j, &mut sum_addr, c_str!("sum"));
//     if !err.is_null() {
//         panic!("Error: {}", CString::from_raw(LLVMGetErrorMessage(err)).into_string().unwrap())
//     }
//
//     println!("Success")
// }