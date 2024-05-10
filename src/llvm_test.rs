use std::ffi::{c_char, CString};
use llvm_sys::core::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0").as_ptr() as *const i8
    );
}

pub unsafe fn llvm_test() {
    let context = LLVMContextCreate();
    let module = LLVMModuleCreateWithNameInContext(c_str!("main"), context);

    let builder = LLVMCreateBuilderInContext(context);

    LLVMSetTarget(module, c_str!("wasm32-unknown-unknown"));

    // LLVMWriteBitcodeToFile(module, c_str!("target/llvm-bitcode.bc"));
    println!("{}", CString::from_raw(LLVMPrintModuleToString(module)).into_string().unwrap());

    LLVMInitializeWebAssemblyTarget();
    LLVMInitializeWebAssemblyTargetInfo();
    LLVMInitializeWebAssemblyTargetMC();
    LLVMInitializeWebAssemblyAsmParser();
    LLVMInitializeWebAssemblyAsmPrinter();
    LLVMInitializeWebAssemblyDisassembler();
    // LLVM_InitializeAllTargets();
    // LLVM_InitializeAllTargetInfos();
    // LLVM_InitializeAllTargetMCs();

    let mut target: LLVMTargetRef = std::ptr::null_mut();
    let mut error_message: *mut c_char = std::ptr::null_mut();
    let has_error = LLVMGetTargetFromTriple(c_str!("wasm32-unknown-unknown"), &mut target, &mut error_message);
    if has_error != 0 {
        panic!("Could not find target: {}", CString::from_raw(error_message).into_string().unwrap());
    }

    let opts = LLVMCreateTargetMachineOptions();
    let target_machine = LLVMCreateTargetMachineWithOptions(target, c_str!("wasm32-unknown-unknown"), opts);

    let has_error = LLVMTargetMachineEmitToFile(
        target_machine,
        module,
        c_str!("target/llvm-out.wasm") as *mut c_char,
        LLVMCodeGenFileType::LLVMObjectFile,
        &mut error_message
    );
    if has_error != 0 {
        panic!("Could not build target object: {}", CString::from_raw(error_message).into_string().unwrap());
    }

    // Needs to be linked with:
    //
    //    wasm-ld --no-entry -o target/llvm-out-linked.wasm target/llvm-out.wasm
    //
    // And can be disassembled with:
    //
    //    wasm2wat -o target/llvm-out-linked.wat target/llvm-out-linked.wasm

    LLVMDisposeBuilder(builder);

    LLVMDisposeModule(module);
    LLVMContextDispose(context);
}
