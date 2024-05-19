use std::collections::HashMap;
use std::ffi::{c_char, c_uint, CString};
use std::ptr;
use llvm_sys::analysis::*;
use llvm_sys::core::*;
use llvm_sys::error::LLVMGetErrorMessage;
use llvm_sys::orc2::*;
use llvm_sys::orc2::lljit::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;
use llvm_sys::transforms::pass_builder::*;
use lib::{Any};
use crate::backend::llvm::host_fn::HostFn;
use crate::backend::llvm::symbol_name_counter::SymbolNameCounter;
use crate::compiler;

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0").as_ptr() as *const i8
    );
}

pub(crate) use c_str;
use crate::backend::llvm::function_builder::FunctionCompiler;
use crate::compiler::mir;

pub struct LLVMJITCompiler<'a> {
    thread_safe_context: LLVMOrcThreadSafeContextRef,
    context: CompilerModuleContext,
    jit: LLVMOrcLLJITRef,

    mir_module: &'a compiler::Module,
    runtime_main_fn: &'a mir::Function
}

pub struct CompilerModuleContext {
    pub context: LLVMContextRef,
    pub module: LLVMModuleRef,

    pub any_t: LLVMTypeRef,

    pub host_fns: HashMap<String, HostFn>,

    pub str_const_name_gen: SymbolNameCounter,
    pub func_name_gen: SymbolNameCounter
}

impl <'a> LLVMJITCompiler<'a> {
    pub fn new(mir_module: &'a compiler::Module) -> Self {
        unsafe {
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();

            let thread_safe_context = LLVMOrcCreateNewThreadSafeContext();
            let llvm_context = LLVMOrcThreadSafeContextGetContext(thread_safe_context);
            let module = LLVMModuleCreateWithNameInContext(c_str!("main"), llvm_context);

            let any_t = LLVMStructCreateNamed(llvm_context, c_str!("Any"));
            LLVMStructSetBody(any_t, [
                LLVMInt32TypeInContext(llvm_context),
                LLVMInt64TypeInContext(llvm_context)
            ].as_mut_ptr(), 2, 0);

            let host_fns = HashMap::from([
                HostFn::new_pair(
                    module,
                    "call",
                    LLVMFunctionType(
                        any_t,
                        [
                            LLVMPointerTypeInContext(llvm_context, 0 as c_uint), // name: *const u8
                            LLVMPointerTypeInContext(llvm_context, 0 as c_uint), // args: *const Value
                            LLVMInt64TypeInContext(llvm_context)                              // arg_count: u64
                        ].as_mut_ptr(),
                        3,
                        0
                    ),
                    runtime::call as *const ()
                ),

                HostFn::new_pair(
                    module,
                    // TODO: Check why we need to do this - are we referencing the real malloc otherwise? What links it?
                    "mallocc",
                    LLVMFunctionType(
                        LLVMPointerTypeInContext(llvm_context, 0 as c_uint),
                        [LLVMInt64TypeInContext(llvm_context)].as_mut_ptr(),
                        1,
                        0
                    ),
                    runtime::malloc as *const ()
                )
            ]);

            let mut jit: LLVMOrcLLJITRef = ptr::null_mut();
            let error_ref = LLVMOrcCreateLLJIT(&mut jit, ptr::null_mut()); // The builder arg can be null here
            if !error_ref.is_null() {
                let error_message = LLVMGetErrorMessage(error_ref);
                panic!("Could not create JIT: {}", CString::from_raw(error_message).into_string().unwrap());
            }

            let runtime_main_fn = &mir_module.runtime_main;

            let context = CompilerModuleContext {
                context: llvm_context,
                module,
                any_t,
                host_fns,
                str_const_name_gen: SymbolNameCounter::new(),
                func_name_gen: SymbolNameCounter::new(),
            };

            LLVMJITCompiler {
                thread_safe_context,
                context,
                jit,
                mir_module,
                runtime_main_fn
            }
        }
    }

    pub fn compile(&'a mut self) -> extern "C" fn() -> Any {
        unsafe {
            // self.compile_fn(&self.mir_module.runtime_main, "main");
            FunctionCompiler::compile(
                &mut self.context,
                self.mir_module,
                self.runtime_main_fn,
                "main",
                false,
                false
            );

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

            std::mem::transmute(self.jit_find_fn_address(self.jit, "main"))
        }
    }

    fn verify_module(&self) {
        unsafe {
            let mut error_message: *mut c_char = ptr::null_mut();
            let result = LLVMVerifyModule(self.context.module, LLVMVerifierFailureAction::LLVMReturnStatusAction, &mut error_message);
            if result != 0 {
                panic!("Generated LLVM module is not valid: {}", CString::from_raw(error_message).into_string().unwrap());
            }
        }
    }

    fn print_module(&self) {
        unsafe {
            println!("{}", CString::from_raw(LLVMPrintModuleToString(self.context.module)).into_string().unwrap());
        }
    }

    unsafe fn jit_compile_module(&self) {
        // https://llvm.org/doxygen/group__LLVMCExecutionEngineORC.html
        let thread_safe_module = LLVMOrcCreateNewThreadSafeModule(self.context.module, self.thread_safe_context);

        let dylib = LLVMOrcLLJITGetMainJITDylib(self.jit);

        let error_ref = LLVMOrcLLJITAddLLVMIRModule(self.jit, dylib, thread_safe_module);
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not add module to JIT: {}", CString::from_raw(error_message).into_string().unwrap());
        }

        let mut host_symbols = Vec::new();

        for (name, intrinsic_fn) in &self.context.host_fns {
            let c_name = CString::new(name.clone()).unwrap();

            let fn_name = LLVMOrcLLJITMangleAndIntern(self.jit, c_name.as_ptr());
            let symbol = LLVMJITEvaluatedSymbol {
                Address: intrinsic_fn.func_addr as u64,
                Flags: LLVMJITSymbolFlags {
                    GenericFlags: 0, // LLVMJITSymbolGenericFlagsExported as u8,
                    TargetFlags: 0, // LLVMJITSymbolGenericFlagsExported as u8
                }
            };

            host_symbols.push(LLVMOrcCSymbolMapPair { Name: fn_name, Sym: symbol })
        }

        let materialization_unit = LLVMOrcAbsoluteSymbols(host_symbols.as_mut_ptr(), host_symbols.len());
        let error_ref = LLVMOrcJITDylibDefine(dylib, materialization_unit);
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not link to parent: {}", CString::from_raw(error_message).into_string().unwrap());
        }
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

        LLVMRunPasses(self.context.module, c_str!("default<O3>"), target_machine, pass_opts);
    }

    unsafe fn jit_find_fn_address(&self, jit: LLVMOrcLLJITRef, name: &str) -> u64 {
        let mut fn_address: LLVMOrcExecutorAddress = 0;

        let c_name = CString::new(name).unwrap();

        let error_ref = LLVMOrcLLJITLookup(jit, &mut fn_address, c_name.as_ptr());
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not look up '{}' function: {}", name, CString::from_raw(error_message).into_string().unwrap());
        }

        fn_address
    }
}