use core::ffi;
use std::ffi::{c_char, c_void, CString};
use std::process::Command;
use std::ptr;
use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMVerifyModule};
use llvm_sys::core::*;
use llvm_sys::debuginfo::LLVMDIBuilderCreateConstantValueExpression;
use llvm_sys::error::{LLVMDisposeErrorMessage, LLVMGetErrorMessage};
use llvm_sys::LLVMLinkage::LLVMExternalLinkage;
use llvm_sys::{LLVMDiagnosticHandler, LLVMVisibility};
use llvm_sys::execution_engine::{LLVMCreateExecutionEngineForModule, LLVMExecutionEngineRef};
use llvm_sys::orc2::lljit::{LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcDisposeLLJITBuilder, LLVMOrcLLJITAddLLVMIRModule, LLVMOrcLLJITGetExecutionSession, LLVMOrcLLJITGetGlobalPrefix, LLVMOrcLLJITGetMainJITDylib, LLVMOrcLLJITLookup, LLVMOrcLLJITMangleAndIntern, LLVMOrcLLJITRef};
use llvm_sys::orc2::{LLVMJITEvaluatedSymbol, LLVMJITSymbolFlags, LLVMJITSymbolTargetFlags, LLVMOrcAbsoluteSymbols, LLVMOrcCJITDylibSearchOrder, LLVMOrcCreateDynamicLibrarySearchGeneratorForProcess, LLVMOrcCreateNewThreadSafeContext, LLVMOrcCreateNewThreadSafeModule, LLVMOrcCSymbolMapPair, LLVMOrcDisposeThreadSafeContext, LLVMOrcExecutionSessionCreateJITDylib, LLVMOrcExecutionSessionIntern, LLVMOrcExecutionSessionLookup, LLVMOrcExecutorAddress, LLVMOrcJITDylibAddGenerator, LLVMOrcJITDylibCreateResourceTracker, LLVMOrcJITDylibDefine, LLVMOrcJITDylibRef, LLVMOrcLookupKind, LLVMOrcThreadSafeContextGetContext};
use llvm_sys::orc2::LLVMJITSymbolGenericFlags::LLVMJITSymbolGenericFlagsExported;
use llvm_sys::prelude::LLVMDiagnosticInfoRef;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;
use llvm_sys::transforms::pass_builder::{LLVMCreatePassBuilderOptions, LLVMPassBuilderOptionsSetDebugLogging, LLVMPassBuilderOptionsSetInlinerThreshold, LLVMRunPasses};

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0").as_ptr() as *const i8
    );
}

pub extern "C" fn diagnostic_handler(arg1: LLVMDiagnosticInfoRef, arg2: *mut ffi::c_void) {
    unsafe {
        let severity = LLVMGetDiagInfoSeverity(arg1);
        let description = LLVMGetDiagInfoDescription(arg1);

        println!("Diagnostic severity={:?} description: {}", severity, CString::from_raw(description).into_string().unwrap())
    }
}

pub extern "C" fn host_add(a: i64, b: i64) -> i64 {
    a + b
}

pub unsafe fn llvm_test() {
    let thread_safe_context = LLVMOrcCreateNewThreadSafeContext();
    let context = LLVMOrcThreadSafeContextGetContext(thread_safe_context);

    // let context = LLVMContextCreate();
    let module = LLVMModuleCreateWithNameInContext(c_str!("main"), context);

    let builder = LLVMCreateBuilderInContext(context);

    // let void_type = LLVMVoidTypeInContext(context);

    let mut host_add_params = [LLVMInt64TypeInContext(context), LLVMInt64TypeInContext(context)];
    let host_add_type = LLVMFunctionType(LLVMInt64TypeInContext(context), host_add_params.as_mut_ptr(), 2, 0);
    let host_add_ref = LLVMAddFunction(module, c_str!("host_add"), host_add_type);


    // `main` function
    {
        // let i32 = LLVMInt32TypeInContext(context);
        let i64 = LLVMInt64TypeInContext(context);

        // let value_type = LLVMStructCreateNamed(context, c_str!("value"));
        // LLVMStructSetBody(value_type, [i32, i64].as_mut_ptr(), 2, 1);

        let main_type = LLVMFunctionType(LLVMInt64TypeInContext(context), ptr::null_mut(), 0, 0);
        let main_func = LLVMAddFunction(module, c_str!("main"), main_type);

        // Do we need this?
        // LLVMSetLinkage(main_func, LLVMExternalLinkage);

        let main_block = LLVMAppendBasicBlockInContext(context, main_func, c_str!("main_block"));
        LLVMPositionBuilderAtEnd(builder, main_block);

        let a = LLVMConstInt(i64, 41, 1);
        let b = LLVMConstInt(i64, 1, 1);

        let mut args = [a, b];
        let result = LLVMBuildCall2(builder, host_add_type, host_add_ref, args.as_mut_ptr(), 2, c_str!("call_host_add"));

        // let result = LLVMBuildAdd(builder, a, b, c_str!("test"));

        // let result_struct = LLVMBuildAlloca(builder, value_type, c_str!("result"));
        //
        // let result_a_ptr = LLVMBuildStructGEP2(builder, value_type, result_struct, 0, c_str!("a"));
        // LLVMBuildStore(builder, a, result_a_ptr);
        //
        // let result_b_ptr = LLVMBuildStructGEP2(builder, value_type, result_struct, 1, c_str!("b"));
        // LLVMBuildStore(builder, b, result_b_ptr);

        // LLVMBuildRet(builder, result);
        // LLVMBuildRetVoid(builder);

        // LLVMSetVisibility(main_func, LLVMVisibility::LLVMDefaultVisibility);
        // LLVMAddTargetDependentFunctionAttr(main_func, c_str!("wasm-export-name"), c_str!("main-export"));
        // LLVMAddTargetDependentFunctionAttr(main_func, c_str!("target-features"), c_str!("+multivalue,+tail-call"));

        // let add_result = LLVMBuildAdd(builder, b, b, c_str!("add_result"));
        LLVMBuildRet(builder, result);

        // let load = LLVMBuildLoad2(builder, value_type, result_struct, c_str!("res"));
        // LLVMBuildRet(builder, load);
    }
    LLVMDisposeBuilder(builder);

    let mut error_message: *mut c_char = ptr::null_mut();
    let result = LLVMVerifyModule(module, LLVMVerifierFailureAction::LLVMReturnStatusAction, &mut error_message);
    if result != 0 {
        panic!("Generated LLVM module is not valid: {}", CString::from_raw(error_message).into_string().unwrap());
    }

    // LLVMSetTarget(module, c_str!("wasm32-unknown-unknown"));

    // LLVMSetDataLayout(module, c_str!("e-m:e-p:32:32-i64:64-n32:64-S128"));

    // LLVMWriteBitcodeToFile(module, c_str!("target/llvm-bitcode.bc"));
    println!("{}", CString::from_raw(LLVMPrintModuleToString(module)).into_string().unwrap());

    LLVM_InitializeAllTargets();
    LLVM_InitializeAllTargetInfos();
    LLVM_InitializeAllTargetMCs();

    LLVM_InitializeNativeTarget();
    LLVM_InitializeNativeAsmPrinter();
    LLVM_InitializeNativeAsmParser();
    LLVM_InitializeNativeDisassembler();

    // WASM generation
    // {
    //     LLVMInitializeWebAssemblyTarget();
    //     LLVMInitializeWebAssemblyTargetInfo();
    //     LLVMInitializeWebAssemblyTargetMC();
    //     LLVMInitializeWebAssemblyAsmParser();
    //     LLVMInitializeWebAssemblyAsmPrinter();
    //     LLVMInitializeWebAssemblyDisassembler();
    //
    //     let mut target: LLVMTargetRef = std::ptr::null_mut();
    //     let mut error_message: *mut c_char = std::ptr::null_mut();
    //     let has_error = LLVMGetTargetFromTriple(c_str!("wasm32-unknown-unknown"), &mut target, &mut error_message);
    //     if has_error != 0 {
    //         panic!("Could not find target: {}", CString::from_raw(error_message).into_string().unwrap());
    //     }
    //
    //     let opts = LLVMCreateTargetMachineOptions();
    //     LLVMTargetMachineOptionsSetCodeGenOptLevel(opts, LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive);
    //     let target_machine = LLVMCreateTargetMachineWithOptions(target, c_str!("wasm32-unknown-unknown"), opts);
    //
    //     // let pass_manager = LLVMCreatePassManager();
    //     // LLVMPassBuilderOptions
    //     // LLVMRunPassManager(pass_manager, module);
    //     let pass_opts = LLVMCreatePassBuilderOptions();
    //     LLVMPassBuilderOptionsSetDebugLogging(pass_opts, 1);
    //     // LLVMPassBuilderOptionsSetInlinerThreshold(pass_opts, 3);
    //
    //     LLVMRunPasses(module, c_str!("default<O3>"), target_machine, pass_opts);
    //
    //     let has_error = LLVMTargetMachineEmitToFile(
    //         target_machine,
    //         module,
    //         c_str!("target/llvm-out.wasm") as *mut c_char,
    //         LLVMCodeGenFileType::LLVMObjectFile,
    //         &mut error_message
    //     );
    //     if has_error != 0 {
    //         panic!("Could not build target object: {}", CString::from_raw(error_message).into_string().unwrap());
    //     }
    //
    //     Command::new("wasm2wat")
    //         .args([
    //             "-o", "target/llvm-out.wat",
    //             "target/llvm-out.wasm"
    //         ])
    //         .spawn()
    //         .expect("Could not get wat")
    //         .wait().expect("Could not link");
    //
    //     Command::new("wasm-ld")
    //         .args([
    //             "--no-entry",
    //             "-o", "target/llvm-out-linked.wasm",
    //             "target/llvm-out.wasm"
    //         ])
    //         .spawn()
    //         .expect("Could not link")
    //         .wait().expect("Could not link");
    //
    //     Command::new("wasm2wat")
    //         .args([
    //             "-o", "target/llvm-out-linked.wat",
    //             "target/llvm-out-linked.wasm"
    //         ])
    //         .spawn()
    //         .expect("Could not get wat")
    //         .wait().expect("Could not link");
    // }

    // JIT
    {
        LLVMContextSetDiagnosticHandler(context, Some(diagnostic_handler), ptr::null_mut());

        // https://llvm.org/doxygen/group__LLVMCExecutionEngineORC.html
        let thread_safe_module = LLVMOrcCreateNewThreadSafeModule(module, thread_safe_context);

        // https://github.com/llvm/llvm-project/blob/main/llvm/examples/OrcV2Examples/OrcV2CBindingsBasicUsage/OrcV2CBindingsBasicUsage.c#L64C3-L66C42
        // LLVMOrcDisposeThreadSafeContext(thread_safe_context);


        // let builder = LLVMOrcCreateLLJITBuilder();

        let mut jit: LLVMOrcLLJITRef = ptr::null_mut();
        let error_ref = LLVMOrcCreateLLJIT(&mut jit, ptr::null_mut()); // The builder arg can be null here
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not create JIT: {}", CString::from_raw(error_message).into_string().unwrap());
        }

        let dylib = LLVMOrcLLJITGetMainJITDylib(jit);
        // let es = LLVMOrcLLJITGetExecutionSession(jit);
        // let mut dylib: LLVMOrcJITDylibRef = ptr::null_mut();
        // LLVMOrcExecutionSessionCreateJITDylib(es, &mut dylib, c_str!("session"));

        let error_ref = LLVMOrcLLJITAddLLVMIRModule(jit, dylib, thread_safe_module);
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not add module to JIT: {}", CString::from_raw(error_message).into_string().unwrap());
        }

        // LLVMOrcCreateDynamicLibrarySearchGeneratorForProcess(search_generator, LLVMOrcLLJITGetGlobalPrefix(jit), )
        // LLVMOrcJITDylibAddGenerator(dylib, );

        // let host_add_name = LLVMOrcExecutionSessionIntern(es, c_str!("_host_add"));
        let host_add_name = LLVMOrcLLJITMangleAndIntern(jit, c_str!("host_add"));
        let host_add_symbol = LLVMJITEvaluatedSymbol {
            Address: host_add as *const () as u64,
            Flags: LLVMJITSymbolFlags {
                GenericFlags: LLVMJITSymbolGenericFlagsExported as u8,
                TargetFlags: LLVMJITSymbolGenericFlagsExported as u8
            }
        };

        let mut symbols = [
            LLVMOrcCSymbolMapPair { Name: host_add_name, Sym: host_add_symbol, }
        ];

        let materialization_unit = LLVMOrcAbsoluteSymbols(symbols.as_mut_ptr(), symbols.len());



        let error_ref = LLVMOrcJITDylibDefine(dylib, materialization_unit);
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not link to parent: {}", CString::from_raw(error_message).into_string().unwrap());
        }

        // let tracker = LLVMOrcJITDylibCreateResourceTracker(dylib);

        // let mut ee: LLVMExecutionEngineRef = ptr::null_mut();
        // LLVMCreateExecutionEngineForModule(&mut ee, module, &mut error_message);

        let mut fn_address: LLVMOrcExecutorAddress = 0;

        // LLVMOrcExecutionSessionLookup(es, LLVMOrcLookupKind::LLVMOrcLookupKindStatic, )

        let error_ref = LLVMOrcLLJITLookup(jit, &mut fn_address, c_str!("main"));
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not look up main function: {}", CString::from_raw(error_message).into_string().unwrap());
        }

        println!("Function address: {}", fn_address);

        let fn_address: fn() -> i64 = std::mem::transmute(fn_address);
        let result = fn_address();

        println!("Got JIT-ed result: {}", result);

        // builder owned by LLVMOrcCreateLLJIT
        // LLVMOrcDisposeLLJITBuilder(builder);
    }

    // LLVMDisposeBuilder(builder);

    // Owned by the LLJIT
    // LLVMDisposeModule(module);
    // LLVMContextDispose(context);
}

#[repr(C, packed)]
struct Value {
    typ: i32,
    val: i64
}

