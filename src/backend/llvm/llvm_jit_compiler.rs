use std::ffi::{c_char, c_uint, c_ulonglong, CString};
use std::ptr;
use llvm_sys::analysis::*;
use llvm_sys::core::*;
use llvm_sys::error::LLVMGetErrorMessage;
use llvm_sys::orc2::*;
use llvm_sys::orc2::lljit::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use lib::Value;
use crate::backend::llvm::anon_id::AnonCounter;
use crate::backend::llvm::ref_table::RefTable;
use crate::compiler;
use crate::compiler::mir;
use crate::compiler::mir::Node;

pub struct LLVMJITCompiler<'a> {
    mir_module: &'a compiler::Module,

    thread_safe_context: LLVMOrcThreadSafeContextRef,
    context: LLVMContextRef,
    module: LLVMModuleRef,

    value_t: LLVMTypeRef
}

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0").as_ptr() as *const i8
    );
}

// macro_rules! c_str_format {
//     ($($arg:tt)*) => (
//         CString::new(format!($($arg)*)).into()
//     );
// }

impl <'a> LLVMJITCompiler<'a> {
    pub fn new(mir_module: &'a compiler::Module) -> Self {
        unsafe {
            let thread_safe_context = LLVMOrcCreateNewThreadSafeContext();
            let context = LLVMOrcThreadSafeContextGetContext(thread_safe_context);
            let module = LLVMModuleCreateWithNameInContext(c_str!("main"), context);

            let value_t = LLVMStructCreateNamed(context, c_str!("Value"));
            LLVMStructSetBody(value_t, [
                LLVMInt32TypeInContext(context),
                LLVMInt64TypeInContext(context)
            ].as_mut_ptr(), 2, 0);

            LLVMJITCompiler {
                mir_module,

                thread_safe_context,
                context,
                module,

                value_t
            }
        }
    }

    pub fn compile(&mut self) -> extern "C" fn() -> Value {
        unsafe {
            self.compile_fn(&self.mir_module.runtime_main, "main");

            let jit = self.jit_compile_module();

            self.print_module();
            self.verify_module();

            std::mem::transmute(self.jit_find_fn_address(jit, "main"))
        }
    }

    unsafe fn compile_fn(&self, func: &mir::Function, name: &str) -> LLVMValueRef {
        let mut param_types = Vec::new();
        for _ in 0..func.param_count {
            param_types.push(self.value_t);
        }

        let fn_type = LLVMFunctionType(self.value_t, param_types.as_mut_ptr(), func.param_count as c_uint, 0);
        // let fn_type = LLVMFunctionType(LLVMVoidTypeInContext(self.context), param_types.as_mut_ptr(), func.param_count as c_uint, 0);

        let fn_name = CString::new(name).unwrap();
        let fn_ref = LLVMAddFunction(self.module, fn_name.as_ptr(), fn_type);

        let entry_block = LLVMAppendBasicBlockInContext(self.context, fn_ref, c_str!("entry"));
        let builder = LLVMCreateBuilderInContext(self.context);

        LLVMPositionBuilderAtEnd(builder, entry_block);

        let mut local_refs = RefTable::new(func.local_count);
        for i in 0..func.local_count {
            let local_name = CString::new(format!("local.{}", i)).unwrap();

            local_refs.table.push(LLVMBuildAlloca(builder, self.value_t, local_name.as_ptr()));
        }

        let mut anon_counter = AnonCounter::new();

        let result = self.compile_mir(func, fn_ref, builder, &mut local_refs, &mut anon_counter, &func.body);
        let result = match result {
            None => self.make_const_value(Value::none()),
            Some(result) => result
        };

        LLVMBuildRet(builder, result);
        // LLVMBuildRetVoid(builder);

        fn_ref
    }

    unsafe fn compile_mir(
        &self,
        func: &mir::Function,
        func_ref: LLVMValueRef,
        builder: LLVMBuilderRef,
        local_refs: &mut RefTable,
        anon_counter: &mut AnonCounter,
        mir: &mir::MIR
    ) -> Option<LLVMValueRef> {
        match &mir.node {
            Node::Nop => Some(self.make_const_value(Value::none())),

            Node::CompileTimeRef(_) => todo!("Support CompileTimeRef"),
            Node::CompileTimeSet(_, _) => todo!("Support CompileTimeSet"),
            Node::GlobalRef(_) => todo!("Support GlobalRef"),
            Node::ConstStringRef(_) => todo!("Support ConstStringRef"),

            Node::LiteralBool(value) => Some(self.make_const_value(Value::bool(*value))),
            Node::LiteralI64(value) => Some(self.make_const_value(Value::int(*value))),
            Node::LiteralF64(value) => Some(self.make_const_value(Value::float(*value))),

            Node::ParamRef(param_ref) => Some(LLVMGetParam(func_ref, param_ref.i as c_uint)),
            Node::CaptureRef(_) => todo!("Support CaptureRef"),

            Node::LocalSet(local_ref, value_mir) => {
                let value_ref = self.compile_mir(func, func_ref, builder, local_refs, anon_counter, value_mir);
                let local_ref = local_refs.table[local_ref.i];

                LLVMBuildStore(builder, self.coalesce_value(value_ref), local_ref);

                None
            },
            Node::LocalGet(local_ref) => {
                let local_ref = local_refs.table[local_ref.i];
                let name = anon_counter.next_str();

                Some(LLVMBuildLoad2(builder, LLVMGetAllocatedType(local_ref), local_ref, name.as_ptr()))
            },

            Node::Block(mirs) => {
                let mut result = None;

                for mir in mirs {
                    result = self.compile_mir(func, func_ref, builder, local_refs, anon_counter, mir);
                }

                result
            },

            Node::Call(_, _, _) => todo!("Support Call"),
            Node::CreateClosure(_, _) => todo!("Support CreateClosure"),
            Node::If(_, _, _) => todo!("Support If")
        }
    }

    unsafe fn make_const_value(&self, value: Value) -> LLVMValueRef {
        let (t, v) = value.into_raw();

        LLVMConstNamedStruct(self.value_t, [
            LLVMConstInt(LLVMInt32TypeInContext(self.context), t as c_ulonglong, 0),
            LLVMConstInt(LLVMInt64TypeInContext(self.context), v as c_ulonglong, 0),
        ].as_mut_ptr(), 2)
    }

    unsafe fn coalesce_value(&self, value_ref: Option<LLVMValueRef>) -> LLVMValueRef {
        match value_ref {
            None => self.make_const_value(Value::none()),
            Some(value_ref) => value_ref
        }
    }

    fn verify_module(&self) {
        unsafe {
            let mut error_message: *mut c_char = ptr::null_mut();
            let result = LLVMVerifyModule(self.module, LLVMVerifierFailureAction::LLVMReturnStatusAction, &mut error_message);
            if result != 0 {
                panic!("Generated LLVM module is not valid: {}", CString::from_raw(error_message).into_string().unwrap());
            }
        }
    }

    fn print_module(&self) {
        unsafe {
            println!("{}", CString::from_raw(LLVMPrintModuleToString(self.module)).into_string().unwrap());
        }
    }

    unsafe fn jit_compile_module(&self) -> LLVMOrcLLJITRef {
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();

        // https://llvm.org/doxygen/group__LLVMCExecutionEngineORC.html
        let thread_safe_module = LLVMOrcCreateNewThreadSafeModule(self.module, self.thread_safe_context);

        let mut jit: LLVMOrcLLJITRef = ptr::null_mut();
        let error_ref = LLVMOrcCreateLLJIT(&mut jit, ptr::null_mut()); // The builder arg can be null here
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not create JIT: {}", CString::from_raw(error_message).into_string().unwrap());
        }

        let dylib = LLVMOrcLLJITGetMainJITDylib(jit);

        let error_ref = LLVMOrcLLJITAddLLVMIRModule(jit, dylib, thread_safe_module);
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not add module to JIT: {}", CString::from_raw(error_message).into_string().unwrap());
        }

        jit
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