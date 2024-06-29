use std::ffi::CString;
use llvm_sys::core::LLVMAddFunction;
use llvm_sys::prelude::{LLVMModuleRef, LLVMTypeRef, LLVMValueRef};

pub struct HostFn {
    pub type_ref: LLVMTypeRef,
    pub func_ref: LLVMValueRef,
    pub func_addr: *const ()
}

impl HostFn {
    pub unsafe fn new(module: LLVMModuleRef, name: &str, type_ref: LLVMTypeRef, func_addr: *const ()) -> Self {
        let name = CString::new(name).unwrap();

        Self {
            type_ref,
            func_ref: LLVMAddFunction(module, name.as_ptr(), type_ref),
            func_addr
        }
    }

    pub unsafe fn new_pair(module: LLVMModuleRef, name: &str, type_ref: LLVMTypeRef, func_addr: *const ()) -> (String, Self) {
        (String::from(name), Self::new(module, name, type_ref, func_addr))
    }
}