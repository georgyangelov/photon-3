use std::ffi::{c_char, CString};
use std::ptr;
use llvm_sys::analysis::*;
use llvm_sys::core::*;
use llvm_sys::error::*;
use llvm_sys::orc2::lljit::*;
use llvm_sys::orc2::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;
use llvm_sys::transforms::pass_builder::*;
use crate::compiler::c_str;
use crate::compiler::compiler::Compiler;
use crate::{ir, lir};

pub struct JITCompiler {
    thread_safe_context: LLVMOrcThreadSafeContextRef,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    jit: LLVMOrcLLJITRef
}

// TODO: Make sure this gets disposed at some point
// impl Drop for JITCompiler {
//     fn drop(&mut self) {
//         unsafe {
//             LLVMOrcDisposeThreadSafeContext(self.thread_safe_context);
//         }
//     }
// }

impl JITCompiler {
    pub fn new() -> Self {
        unsafe {
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();

            let thread_safe_context = LLVMOrcCreateNewThreadSafeContext();
            let context = LLVMOrcThreadSafeContextGetContext(thread_safe_context);
            let module = LLVMModuleCreateWithNameInContext(c_str!("main"), context);

            let mut jit: LLVMOrcLLJITRef = ptr::null_mut();
            let error_ref = LLVMOrcCreateLLJIT(&mut jit, ptr::null_mut()); // The builder arg can be null here
            if !error_ref.is_null() {
                let error_message = LLVMGetErrorMessage(error_ref);
                panic!("Could not create JIT: {}", CString::from_raw(error_message).into_string().unwrap());
            }

            Self {
                thread_safe_context,
                context,
                module,
                jit
            }
        }
    }

    pub fn compile<T>(&mut self, lir_module: &lir::Module) -> unsafe extern "C" fn() -> T {
        Compiler::compile(
            self.context,
            self.module,
            lir_module
        );

        unsafe {
            self.jit_compile_module();

            println!("Before optimization");
            println!("-------------------");
            println!();

            self.print_module();
            self.verify_module();

            println!("After optimization");
            println!("------------------");
            println!();

            self.optimize_module();
            self.print_module();
            println!();
            println!();

            std::mem::transmute(self.jit_find_symbol_address("main"))
        }
    }

    fn print_module(&self) {
        unsafe {
            println!("{}", CString::from_raw(LLVMPrintModuleToString(self.module)).into_string().unwrap());
        }
    }

    unsafe fn jit_compile_module(&self) {
        // https://llvm.org/doxygen/group__LLVMCExecutionEngineORC.html
        let thread_safe_module = LLVMOrcCreateNewThreadSafeModule(self.module, self.thread_safe_context);

        let dylib = LLVMOrcLLJITGetMainJITDylib(self.jit);

        let error_ref = LLVMOrcLLJITAddLLVMIRModule(self.jit, dylib, thread_safe_module);
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not add module to JIT: {}", CString::from_raw(error_message).into_string().unwrap());
        }

        // TODO: Define rust intrinsic methods to be able to call
        // let mut host_symbols = Vec::new();
        //
        // for (name, intrinsic_fn) in &self.context.host_fns {
        //     let c_name = CString::new(name.clone()).unwrap();
        //
        //     let fn_name = LLVMOrcLLJITMangleAndIntern(self.jit, c_name.as_ptr());
        //     let symbol = LLVMJITEvaluatedSymbol {
        //         Address: intrinsic_fn.func_addr as u64,
        //         Flags: LLVMJITSymbolFlags {
        //             GenericFlags: 0, // LLVMJITSymbolGenericFlagsExported as u8,
        //             TargetFlags: 0, // LLVMJITSymbolGenericFlagsExported as u8
        //         }
        //     };
        //
        //     host_symbols.push(LLVMOrcCSymbolMapPair { Name: fn_name, Sym: symbol })
        // }
        //
        // let materialization_unit = LLVMOrcAbsoluteSymbols(host_symbols.as_mut_ptr(), host_symbols.len());
        // let error_ref = LLVMOrcJITDylibDefine(dylib, materialization_unit);
        // if !error_ref.is_null() {
        //     let error_message = LLVMGetErrorMessage(error_ref);
        //     panic!("Could not link to parent: {}", CString::from_raw(error_message).into_string().unwrap());
        // }
    }

    unsafe fn optimize_module(&self) {
        let triple = LLVMOrcLLJITGetTripleString(self.jit);

        let mut target: LLVMTargetRef = ptr::null_mut();
        let mut error_message: *mut c_char = ptr::null_mut();
        let has_error = LLVMGetTargetFromTriple(triple, &mut target, &mut error_message);
        if has_error != 0 {
            panic!("Could not find target: {}", CString::from_raw(error_message).into_string().unwrap());
        }

        let opts = LLVMCreateTargetMachineOptions();
        LLVMTargetMachineOptionsSetCodeGenOptLevel(opts, LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive);

        let pass_opts = LLVMCreatePassBuilderOptions();
        // LLVMPassBuilderOptionsSetDebugLogging(pass_opts, 1);

        let target_machine = LLVMCreateTargetMachineWithOptions(target, triple, opts);

        LLVMRunPasses(self.module, c_str!("default<O3>"), target_machine, pass_opts);
    }

    unsafe fn jit_find_symbol_address(&self, name: &str) -> u64 {
        let mut symbol_address: LLVMOrcExecutorAddress = 0;

        let c_name = CString::new(name).unwrap();

        let error_ref = LLVMOrcLLJITLookup(self.jit, &mut symbol_address, c_name.as_ptr());
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not look up '{}' symbol: {}", name, CString::from_raw(error_message).into_string().unwrap());
        }

        symbol_address
    }

    // TODO: Move this to the Compiler impl
    fn verify_module(&self) {
        unsafe {
            let mut error_message: *mut c_char = ptr::null_mut();
            let result = LLVMVerifyModule(
                self.module,
                LLVMVerifierFailureAction::LLVMReturnStatusAction,
                &mut error_message
            );
            if result != 0 {
                panic!("Generated LLVM module is not valid: {}", CString::from_raw(error_message).into_string().unwrap());
            }
        }
    }
}