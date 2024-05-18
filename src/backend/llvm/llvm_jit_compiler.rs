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
use lib::{Any, AnyT};
use crate::backend::llvm::host_fn::HostFn;
use crate::backend::llvm::symbol_name_counter::SymbolNameCounter;
use crate::backend::llvm::ref_table::RefTable;
use crate::compiler;
use crate::compiler::lexical_scope::{CaptureFrom, CaptureRef, StackFrameLocalRef};
use crate::compiler::mir;
use crate::compiler::mir::Node;

pub struct LLVMJITCompiler<'a> {
    mir_module: &'a compiler::Module,

    thread_safe_context: LLVMOrcThreadSafeContextRef,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    jit: LLVMOrcLLJITRef,

    any_t: LLVMTypeRef,

    host_fns: HashMap<String, HostFn>,
    
    str_const_name_gen: SymbolNameCounter,
    func_name_gen: SymbolNameCounter
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

    capture_arg_index: Option<usize>,
    capture_struct_type: Option<LLVMTypeRef>
}

impl <'a> LLVMJITCompiler<'a> {
    pub fn new(mir_module: &'a compiler::Module) -> Self {
        unsafe {
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();

            let thread_safe_context = LLVMOrcCreateNewThreadSafeContext();
            let context = LLVMOrcThreadSafeContextGetContext(thread_safe_context);
            let module = LLVMModuleCreateWithNameInContext(c_str!("main"), context);

            let value_t = LLVMStructCreateNamed(context, c_str!("Any"));
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
                ),

                HostFn::new_pair(
                    module,
                    // TODO: Check why we need to do this - are we referencing the real malloc otherwise? What links it?
                    "mallocc",
                    LLVMFunctionType(
                        LLVMPointerTypeInContext(context, 0 as c_uint),
                        [LLVMInt64TypeInContext(context)].as_mut_ptr(),
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

            LLVMJITCompiler {
                mir_module,

                thread_safe_context,
                context,
                module,
                jit,

                any_t: value_t,

                host_fns,
                str_const_name_gen: SymbolNameCounter::new(),
                func_name_gen: SymbolNameCounter::new(),
            }
        }
    }

    pub fn compile(&mut self) -> extern "C" fn() -> Any {
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

    unsafe fn compile_fn(&mut self, func: &mir::Function, name: &str) -> (LLVMTypeRef, LLVMValueRef, Option<LLVMTypeRef>) {
        let mut param_types = Vec::with_capacity(func.param_count + 1);
        for _ in 0..func.param_count {
            param_types.push(self.any_t);
        }

        let has_capture_struct = func.captures.len() > 0;
        if has_capture_struct {
            param_types.push(LLVMPointerTypeInContext(self.context, 0 as c_uint));
        }

        let fn_type = LLVMFunctionType(self.any_t, param_types.as_mut_ptr(), param_types.len() as c_uint, 0);
        // let fn_type = LLVMFunctionType(LLVMVoidTypeInContext(self.context), param_types.as_mut_ptr(), func.param_count as c_uint, 0);

        let fn_name = CString::new(name).unwrap();
        let func_ref = LLVMAddFunction(self.module, fn_name.as_ptr(), fn_type);

        let entry_block = LLVMAppendBasicBlockInContext(self.context, func_ref, c_str!("entry"));
        let builder = LLVMCreateBuilderInContext(self.context);

        LLVMPositionBuilderAtEnd(builder, entry_block);

        let mut local_refs = RefTable::new(func.local_count);
        for i in 0..func.local_count {
            let local_name = CString::new(format!("local.{}", i)).unwrap();

            local_refs.table.push(LLVMBuildAlloca(builder, self.any_t, local_name.as_ptr()));
        }

        let capture_struct_type = if has_capture_struct {
            Some(self.closure_struct_type(func.captures.len()))
        } else { None };

        let capture_arg_index = if has_capture_struct {
            Some(func.param_count)
        } else { None };

        let mut function_builder = FunctionBuilder {
            local_refs,
            
            stmt_name_gen: SymbolNameCounter::new(),

            func_ref,
            builder,

            capture_arg_index,
            capture_struct_type,
        };

        let result = self.compile_mir(func, &mut function_builder, &func.body);
        let result = match result {
            None => self.make_const_any(Any::none()),
            Some(result) => result
        };

        LLVMBuildRet(builder, result);
        // LLVMBuildRetVoid(builder);

        (fn_type, func_ref, capture_struct_type)
    }

    unsafe fn compile_mir(
        &mut self,
        func: &mir::Function,
        fb: &mut FunctionBuilder,
        mir: &mir::MIR
    ) -> Option<LLVMValueRef> {
        match &mir.node {
            Node::Nop => Some(self.make_const_any(Any::none())),

            Node::CompileTimeRef(_) => todo!("Support CompileTimeRef"),
            Node::CompileTimeSet(_, _) => todo!("Support CompileTimeSet"),
            Node::GlobalRef(_) => todo!("Support GlobalRef"),
            Node::ConstStringRef(_) => todo!("Support ConstStringRef"),

            Node::LiteralBool(value) => Some(self.make_const_any(Any::bool(*value))),
            Node::LiteralI64(value) => Some(self.make_const_any(Any::int(*value))),
            Node::LiteralF64(value) => Some(self.make_const_any(Any::float(*value))),

            Node::ParamRef(param_ref) => Some(LLVMGetParam(fb.func_ref, param_ref.i as c_uint)),
            Node::CaptureRef(capture_ref) => Some(self.build_load_capture(fb, capture_ref)),

            // TODO: Make these use IR registers instead of alloca, or make sure the register local optimization pass is run
            Node::LocalSet(local_ref, value_mir) => {
                let value_ref = self.compile_mir(func, fb, value_mir);
                let local_ref = fb.local_refs.table[local_ref.i];

                LLVMBuildStore(fb.builder, self.coalesce_any(value_ref), local_ref);

                None
            },
            Node::LocalGet(local_ref) => {
                Some(self.build_load_local(fb, local_ref))
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
                args.push(self.coalesce_any(target_ref));

                for arg_mir in arg_mirs {
                    let value_ref = self.compile_mir(func, fb, arg_mir);

                    args.push(self.coalesce_any(value_ref));
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

            Node::CreateClosure(mir_func_ref, captures) => {
                // TODO: Infer the function name from the assignment
                let func_name = self.func_name_gen.next_string("fn");
                let func = &self.mir_module.functions[mir_func_ref.i];

                let (func_type, func_ref, closure_struct_type) = self.compile_fn(func, &func_name);
                let trampoline_func_ref = self.compile_fn_trampoline(func, &func_name, func_type, func_ref, closure_struct_type.is_some());

                match closure_struct_type {
                    None => {
                        let result = self.build_ptr_any(fb, AnyT::FunctionPtr, trampoline_func_ref);

                        Some(result)
                    },
                    Some(closure_t) => {
                        let closure_ptr = self.build_malloc(fb, LLVMSizeOf(closure_struct_type.unwrap()));

                        let ptr = self.build_gep(fb, closure_struct_type.unwrap(), closure_ptr, &mut [
                            self.const_u64(0)
                        ]);
                        // LLVMBuildStore(fb.builder, func_ref, ptr);
                        LLVMBuildStore(fb.builder, trampoline_func_ref, ptr);

                        for (i, capture) in captures.iter().enumerate() {
                            let captured_value_ref = match capture {
                                CaptureFrom::Capture(_) => todo!("Support capture captures"),
                                CaptureFrom::Param(_) => todo!("Support param captures"),
                                CaptureFrom::Local(local_ref) => self.build_load_local(fb, local_ref)
                            };

                            let ptr = self.build_gep(fb, closure_t, closure_ptr, &mut [
                                self.const_i32(0),
                                self.const_i32((i + 1) as i32),
                            ]);
                            LLVMBuildStore(fb.builder, captured_value_ref, ptr);
                        }

                        let result = self.build_ptr_any(fb, AnyT::Closure, closure_ptr);

                        Some(result)
                    }
                }
            },

            Node::If(_, _, _) => todo!("Support If")
        }
    }

    unsafe fn compile_fn_trampoline(
        &self,
        func: &mir::Function,
        fn_name: &str,
        compiled_fn_type: LLVMTypeRef,
        compiled_ref: LLVMValueRef,
        has_capture_struct: bool
    ) -> LLVMValueRef {
        let c_name = CString::new(format!("{}_trampoline", fn_name)).unwrap();
        let mut args = Vec::with_capacity(3);
        args.push(LLVMPointerTypeInContext(self.context, 0 as c_uint)); // args: *const Value

        // TODO: Verify the arg count before calling
        // LLVMInt64TypeInContext(self.context)                              // arg_count: u64

        if has_capture_struct {
            args.push(LLVMPointerTypeInContext(self.context, 0 as c_uint)); // captures: *const <capture struct>
        }

        let trampoline_t = LLVMFunctionType(self.any_t, args.as_mut_ptr(), args.len() as u32, 0);
        let trampoline_fn_ref = LLVMAddFunction(self.module, c_name.as_ptr(), trampoline_t);

        let builder = LLVMCreateBuilderInContext(self.context);
        let block = LLVMAppendBasicBlockInContext(self.context, trampoline_fn_ref, c_str!("entry"));

        LLVMPositionBuilderAtEnd(builder, block);

        let array_args_ref = LLVMGetParam(trampoline_fn_ref, 0);

        let mut args = Vec::with_capacity(func.param_count + 1);
        for i in 0..func.param_count {
            let name = CString::new(format!("arg_ptr.{}", i)).unwrap();
            let ptr = LLVMBuildGEP2(builder, self.any_t, array_args_ref, [
                self.const_u64(i as u64)
            ].as_mut_ptr(), 1, name.as_ptr());

            let name = CString::new(format!("arg.{}", i)).unwrap();

            args.push(LLVMBuildLoad2(builder, self.any_t, ptr, name.as_ptr()));
        }

        if has_capture_struct {
            args.push(LLVMGetParam(trampoline_fn_ref, 1));
        }

        let result = LLVMBuildCall2(
            builder,
            compiled_fn_type,
            compiled_ref,
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!("result")
        );

        LLVMBuildRet(builder, result);

        LLVMDisposeBuilder(builder);

        trampoline_fn_ref
    }

    unsafe fn build_ptr_any(&self, fb: &mut FunctionBuilder, typ: AnyT, ptr_value_ref: LLVMValueRef) -> LLVMValueRef {
        let name = fb.stmt_name_gen.next("value");
        let value_ref = LLVMBuildAlloca(fb.builder, self.any_t, name.as_ptr());

        let ptr = self.build_gep(fb, self.any_t, value_ref, &mut [self.const_i32(0), self.const_i32(0)]);
        LLVMBuildStore(fb.builder, self.const_i32(std::mem::transmute(typ)), ptr);

        let ptr = self.build_gep(fb, self.any_t, value_ref, &mut [self.const_i32(0), self.const_i32(1)]);

        let name = fb.stmt_name_gen.next("ptr_to_int");
        let ptr_value_int_ref = LLVMBuildPtrToInt(fb.builder, ptr_value_ref, LLVMInt64TypeInContext(self.context), name.as_ptr());
        LLVMBuildStore(fb.builder, ptr_value_int_ref, ptr);

        let name = fb.stmt_name_gen.next("value_load");
        LLVMBuildLoad2(fb.builder, self.any_t, value_ref, name.as_ptr())
    }

    unsafe fn build_load_local(&self, fb: &mut FunctionBuilder, local_ref: &StackFrameLocalRef) -> LLVMValueRef {
        let local_ref = fb.local_refs.table[local_ref.i];
        let name = fb.stmt_name_gen.next("local_get");

        LLVMBuildLoad2(fb.builder, LLVMGetAllocatedType(local_ref), local_ref, name.as_ptr())
    }

    unsafe fn build_load_capture(
        &self,
        fb: &mut FunctionBuilder,
        capture_ref: &CaptureRef
    ) -> LLVMValueRef {
        let mut indices = [self.const_i32(0), self.const_i32(capture_ref.i as i32 + 1)];
        let ptr = self.build_gep(
            fb,
            fb.capture_struct_type.expect("Requires capture struct type"),
            LLVMGetParam(fb.func_ref, fb.capture_arg_index.expect("Requires capture_arg_index") as c_uint),
            &mut indices // The +1 is because the first element of the struct is the function pointer
        );

        let name = fb.stmt_name_gen.next("capture_get");
        LLVMBuildLoad2(fb.builder, self.any_t, ptr, name.as_ptr())
    }

    unsafe fn build_gep(&self, fb: &mut FunctionBuilder, type_ref: LLVMTypeRef, ptr_ref: LLVMValueRef, indices: &mut [LLVMValueRef]) -> LLVMValueRef {
        let name = fb.stmt_name_gen.next("struct_ptr");

        LLVMBuildGEP2(fb.builder, type_ref, ptr_ref, indices.as_mut_ptr(), indices.len() as u32, name.as_ptr())
    }

    unsafe fn build_malloc(&self, fb: &mut FunctionBuilder, size: LLVMValueRef) -> LLVMValueRef {
        let malloc = &self.host_fns["mallocc"];
        let name = fb.stmt_name_gen.next("malloc");

        LLVMBuildCall2(fb.builder, malloc.type_ref, malloc.func_ref, [size].as_mut_ptr(), 1, name.as_ptr())
    }

    unsafe fn closure_struct_type(&self, capture_count: usize) -> LLVMTypeRef {
        let mut fields = Vec::with_capacity(1 + capture_count);

        // Function pointer
        fields.push(LLVMPointerTypeInContext(self.context, 0));

        for _ in 0..capture_count {
            fields.push(self.any_t);
        }

        LLVMStructTypeInContext(self.context, fields.as_mut_ptr(), fields.len() as u32, 0)
    }

    unsafe fn const_u64(&self, value: u64) -> LLVMValueRef {
        LLVMConstInt(LLVMInt64TypeInContext(self.context), value, 0)
    }

    unsafe fn const_i32(&self, value: i32) -> LLVMValueRef {
        LLVMConstInt(LLVMInt32TypeInContext(self.context), value as u64, 1)
    }

    unsafe fn build_args_array(&self, fb: &mut FunctionBuilder, args: Vec<LLVMValueRef>) -> (LLVMValueRef, u64) {
        let arg_array_type = LLVMArrayType2(self.any_t, args.len() as u64);

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

    unsafe fn make_const_any(&self, value: Any) -> LLVMValueRef {
        let (t, v) = value.into_raw();

        LLVMConstNamedStruct(self.any_t, [
            LLVMConstInt(LLVMInt32TypeInContext(self.context), t as c_ulonglong, 0),
            LLVMConstInt(LLVMInt64TypeInContext(self.context), v as c_ulonglong, 0),
        ].as_mut_ptr(), 2)
    }

    unsafe fn coalesce_any(&self, value_ref: Option<LLVMValueRef>) -> LLVMValueRef {
        match value_ref {
            None => self.make_const_any(Any::none()),
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