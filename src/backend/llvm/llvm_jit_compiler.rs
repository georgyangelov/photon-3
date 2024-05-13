use std::collections::HashMap;
use std::ffi::{c_char, c_uint, c_ulonglong, CString};
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
use lib::Value;
use crate::backend::llvm::extern_fn::HostFn;
use crate::backend::llvm::symbol_name_counter::SymbolNameCounter;
use crate::backend::llvm::ref_table::RefTable;
use crate::compiler;
use crate::compiler::mir;
use crate::compiler::mir::Node;

pub struct LLVMJITCompiler<'a> {
    mir_module: &'a compiler::Module,

    thread_safe_context: LLVMOrcThreadSafeContextRef,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    jit: LLVMOrcLLJITRef,

    value_t: LLVMTypeRef,

    host_fns: HashMap<String, HostFn>,
    
    str_const_name_gen: SymbolNameCounter
}

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0").as_ptr() as *const i8
    );
}

pub struct FunctionBuilder {
    local_refs: RefTable,

    stmt_name_gen: SymbolNameCounter,

    func_ref: LLVMValueRef,
    builder: LLVMBuilderRef,
}

impl <'a> LLVMJITCompiler<'a> {
    pub fn new(mir_module: &'a compiler::Module) -> Self {
        unsafe {
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();

            let thread_safe_context = LLVMOrcCreateNewThreadSafeContext();
            let context = LLVMOrcThreadSafeContextGetContext(thread_safe_context);
            let module = LLVMModuleCreateWithNameInContext(c_str!("main"), context);

            let value_t = LLVMStructCreateNamed(context, c_str!("Value"));
            LLVMStructSetBody(value_t, [
                LLVMInt32TypeInContext(context),
                LLVMInt64TypeInContext(context)
            ].as_mut_ptr(), 2, 0);

            let host_fns = HashMap::from([
                HostFn::new_pair(
                    module,
                    "call",
                    LLVMFunctionType(
                        value_t,
                        [
                            LLVMPointerTypeInContext(context, 0 as c_uint), // name: *const u8
                            LLVMPointerTypeInContext(context, 0 as c_uint), // args: *const Value
                            LLVMInt64TypeInContext(context)                              // arg_count: u64
                        ].as_mut_ptr(),
                        3,
                        0
                    ),
                    runtime::call as *const ()
                )
            ]);

            let mut jit: LLVMOrcLLJITRef = ptr::null_mut();
            let error_ref = LLVMOrcCreateLLJIT(&mut jit, ptr::null_mut()); // The builder arg can be null here
            if !error_ref.is_null() {
                let error_message = LLVMGetErrorMessage(error_ref);
                panic!("Could not create JIT: {}", CString::from_raw(error_message).into_string().unwrap());
            }

            LLVMJITCompiler {
                mir_module,

                thread_safe_context,
                context,
                module,
                jit,

                value_t,

                host_fns,
                str_const_name_gen: SymbolNameCounter::new(),
            }
        }
    }

    pub fn compile(&mut self) -> extern "C" fn() -> Value {
        unsafe {
            self.compile_fn(&self.mir_module.runtime_main, "main");

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

    unsafe fn compile_fn(&mut self, func: &mir::Function, name: &str) -> LLVMValueRef {
        let mut param_types = Vec::new();
        for _ in 0..func.param_count {
            param_types.push(self.value_t);
        }

        let fn_type = LLVMFunctionType(self.value_t, param_types.as_mut_ptr(), func.param_count as c_uint, 0);
        // let fn_type = LLVMFunctionType(LLVMVoidTypeInContext(self.context), param_types.as_mut_ptr(), func.param_count as c_uint, 0);

        let fn_name = CString::new(name).unwrap();
        let func_ref = LLVMAddFunction(self.module, fn_name.as_ptr(), fn_type);

        let entry_block = LLVMAppendBasicBlockInContext(self.context, func_ref, c_str!("entry"));
        let builder = LLVMCreateBuilderInContext(self.context);

        LLVMPositionBuilderAtEnd(builder, entry_block);

        let mut local_refs = RefTable::new(func.local_count);
        for i in 0..func.local_count {
            let local_name = CString::new(format!("local.{}", i)).unwrap();

            local_refs.table.push(LLVMBuildAlloca(builder, self.value_t, local_name.as_ptr()));
        }

        let mut function_builder = FunctionBuilder {
            local_refs,
            
            stmt_name_gen: SymbolNameCounter::new(),

            func_ref,
            builder,
        };

        let result = self.compile_mir(func, &mut function_builder, &func.body);
        let result = match result {
            None => self.make_const_value(Value::none()),
            Some(result) => result
        };

        LLVMBuildRet(builder, result);
        // LLVMBuildRetVoid(builder);

        func_ref
    }

    unsafe fn compile_mir(
        &mut self,
        func: &mir::Function,
        fb: &mut FunctionBuilder,
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

            Node::ParamRef(param_ref) => Some(LLVMGetParam(fb.func_ref, param_ref.i as c_uint)),
            Node::CaptureRef(_) => todo!("Support CaptureRef"),

            // TODO: Make these use IR registers instead of alloca
            Node::LocalSet(local_ref, value_mir) => {
                let value_ref = self.compile_mir(func, fb, value_mir);
                let local_ref = fb.local_refs.table[local_ref.i];

                LLVMBuildStore(fb.builder, self.coalesce_value(value_ref), local_ref);

                None
            },
            Node::LocalGet(local_ref) => {
                let local_ref = fb.local_refs.table[local_ref.i];
                let name = fb.stmt_name_gen.next("local_get");

                Some(LLVMBuildLoad2(fb.builder, LLVMGetAllocatedType(local_ref), local_ref, name.as_ptr()))
            },

            Node::Block(mirs) => {
                let mut result = None;

                for mir in mirs {
                    result = self.compile_mir(func, fb, mir);
                }

                result
            },

            Node::Call(name, target_mir, arg_mirs) => {
                let mut args = Vec::with_capacity(arg_mirs.len() + 1);

                let target_ref = self.compile_mir(func, fb, target_mir);
                args.push(self.coalesce_value(target_ref));

                for arg_mir in arg_mirs {
                    let value_ref = self.compile_mir(func, fb, arg_mir);

                    args.push(self.coalesce_value(value_ref));
                }

                let (args_array_ref, arg_count) = self.build_args_array(fb, args);

                let mut args_array = [
                    self.build_str_global_const_ref(fb.builder, name),
                    args_array_ref,
                    self.const_u64(arg_count)
                ];

                let call_fn = &self.host_fns["call"];

                let call_name = fb.stmt_name_gen.next("result");
                let call = LLVMBuildCall2(
                    fb.builder,
                    call_fn.type_ref,
                    call_fn.func_ref,
                    args_array.as_mut_ptr(),
                    args_array.len() as c_uint,
                    call_name.as_ptr()
                );

                Some(call)
            }

            Node::CreateClosure(_, _) => todo!("Support CreateClosure"),
            Node::If(_, _, _) => todo!("Support If")
        }
    }

    unsafe fn const_u64(&self, value: u64) -> LLVMValueRef {
        LLVMConstInt(LLVMInt64TypeInContext(self.context), value, 0)
    }

    unsafe fn build_args_array(&self, fb: &mut FunctionBuilder, args: Vec<LLVMValueRef>) -> (LLVMValueRef, u64) {
        let arg_array_type = LLVMArrayType2(self.value_t, args.len() as u64);

        let args_array_ref_name = fb.stmt_name_gen.next("args");
        let args_array_ref = LLVMBuildAlloca(fb.builder, arg_array_type, args_array_ref_name.as_ptr());

        let arg_count = args.len();

        for (i, arg_ref) in args.into_iter().enumerate() {
            let n = fb.stmt_name_gen.next("arg");
            let ptr_to_args_array_element = LLVMBuildGEP2(
                fb.builder,
                arg_array_type,
                args_array_ref,
                [
                    self.const_u64(0),
                    self.const_u64(i as u64)
                ].as_mut_ptr(),
                2,
                n.as_ptr()
            );
            LLVMBuildStore(fb.builder, arg_ref, ptr_to_args_array_element);
        }

        (args_array_ref, arg_count as u64)
    }

    unsafe fn build_str_global_const_ref(&mut self, builder: LLVMBuilderRef, str: &str) -> LLVMValueRef {
        let name_ref_name = self.str_const_name_gen.next("str");
        let c_name = CString::new(str.as_bytes()).unwrap();

        LLVMBuildGlobalStringPtr(builder, c_name.as_ptr(), name_ref_name.as_ptr())
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

    unsafe fn jit_compile_module(&self) {
        // https://llvm.org/doxygen/group__LLVMCExecutionEngineORC.html
        let thread_safe_module = LLVMOrcCreateNewThreadSafeModule(self.module, self.thread_safe_context);

        let dylib = LLVMOrcLLJITGetMainJITDylib(self.jit);

        let error_ref = LLVMOrcLLJITAddLLVMIRModule(self.jit, dylib, thread_safe_module);
        if !error_ref.is_null() {
            let error_message = LLVMGetErrorMessage(error_ref);
            panic!("Could not add module to JIT: {}", CString::from_raw(error_message).into_string().unwrap());
        }

        let mut host_symbols = Vec::new();

        for (name, intrinsic_fn) in &self.host_fns {
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

        LLVMRunPasses(self.module, c_str!("default<O3>"), target_machine, pass_opts);
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