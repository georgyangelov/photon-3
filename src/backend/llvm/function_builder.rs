use std::ffi::{c_uint, c_ulonglong, CString};
use llvm_sys::core::*;
use llvm_sys::LLVMLinkage;
use llvm_sys::prelude::*;
use lib::{Any, AnyT};
use crate::backend::llvm::{c_str, CompilerModuleContext};
use crate::backend::llvm::symbol_name_counter::SymbolNameCounter;
use crate::compiler;
use crate::compiler::lexical_scope::{CaptureFrom, CaptureRef, ParamRef, StackFrameLocalRef};
use crate::compiler::mir;
use crate::compiler::mir::Node;

pub struct FunctionCompiler<'a> {
    local_refs: Vec<Option<LLVMValueRef>>,

    stmt_name_gen: SymbolNameCounter,

    c: &'a mut CompilerModuleContext,

    mir_module: &'a compiler::Module,
    func: &'a mir::Function,
    func_ref: LLVMValueRef,
    builder: LLVMBuilderRef,

    capture_arg_index: Option<usize>,
    capture_struct_type: Option<LLVMTypeRef>
}

pub struct FunctionCompileResult {
    pub type_ref: LLVMTypeRef,
    pub func_ref: LLVMValueRef,
    pub closure_t: Option<LLVMTypeRef>,

    pub trampoline_ref: Option<LLVMValueRef>,
}

impl <'a> FunctionCompiler<'a> {
    pub unsafe fn compile(
        c: &'a mut CompilerModuleContext,
        mir_module: &'a compiler::Module,
        func: &'a mir::Function,
        name: &str,
        with_trampoline: bool,
        internal: bool
    ) -> FunctionCompileResult {
        let mut param_types = Vec::with_capacity(func.param_count + 1);
        for _ in 0..func.param_count {
            param_types.push(c.any_t);
        }

        let has_captures = func.captures.len() > 0;
        if has_captures {
            param_types.push(LLVMPointerTypeInContext(c.context, 0 as c_uint));
        }

        let type_ref = LLVMFunctionType(c.any_t, param_types.as_mut_ptr(), param_types.len() as c_uint, 0);

        let fn_name = CString::new(name).unwrap();
        let func_ref = LLVMAddFunction(c.module, fn_name.as_ptr(), type_ref);

        // LLVMAddAttributeAtIndex(func_ref, 0, LLVMInternal)
        if internal {
            LLVMSetLinkage(func_ref, LLVMLinkage::LLVMInternalLinkage);
        }

        let entry_block = LLVMAppendBasicBlockInContext(c.context, func_ref, c_str!("entry"));
        let builder = LLVMCreateBuilderInContext(c.context);

        LLVMPositionBuilderAtEnd(builder, entry_block);

        let mut local_refs = Vec::with_capacity(func.local_count);
        local_refs.resize(func.local_count, None);

        let closure_t = if has_captures {
            Some(Self::closure_struct_type(c, func.captures.len()))
        } else { None };

        let capture_arg_index = if has_captures {
            Some(func.param_count)
        } else { None };

        let mut function_builder = FunctionCompiler {
            local_refs,

            stmt_name_gen: SymbolNameCounter::new(),

            c,

            mir_module,
            func,
            func_ref,
            builder,

            capture_arg_index,
            capture_struct_type: closure_t,
        };

        let result = function_builder.compile_mir(&func.body);
        let result = match result {
            None => function_builder.build_const_any(Any::none()),
            Some(result) => result
        };

        LLVMBuildRet(builder, result);

        let trampoline_ref = if with_trampoline {
            Some(function_builder.compile_fn_trampoline(name, type_ref, func_ref, closure_t.is_some(), internal))
        } else { None };

        LLVMDisposeBuilder(builder);

        FunctionCompileResult {
            type_ref,
            func_ref,
            closure_t,
            trampoline_ref,
        }
    }

    /// Compiles a single statement.
    /// Returns an LLVMValueRef if the statement returns a value
    unsafe fn compile_mir(&mut self, mir: &mir::MIR) -> Option<LLVMValueRef> {
        match &mir.node {
            Node::Nop => None,

            Node::CompileTimeRef(_) => todo!("Support CompileTimeRef"),
            Node::CompileTimeSet(_, _) => todo!("Support CompileTimeSet"),
            Node::GlobalRef(_) => todo!("Support GlobalRef"),
            Node::ConstStringRef(_) => todo!("Support ConstStringRef"),

            // TODO: For these 3 define a parameter "expected_type" and build them in whichever type (e.g. Any or Int)
            //       is necessary.
            Node::LiteralBool(value) => Some(self.build_const_any(Any::bool(*value))),
            Node::LiteralI64(value) => Some(self.build_const_any(Any::int(*value))),
            Node::LiteralF64(value) => Some(self.build_const_any(Any::float(*value))),

            Node::ParamRef(param_ref) => Some(self.build_param_load(param_ref)),
            Node::CaptureRef(capture_ref) => Some(self.build_capture_load(capture_ref)),

            Node::LocalSet(local_ref, value_mir) => {
                let value_ref = self.compile_mir(value_mir);
                let value_ref = self.coalesce_none(value_ref);

                self.build_local_store(local_ref, value_ref);

                None
            },
            Node::LocalGet(local_ref) => Some(self.build_local_load(local_ref)),

            Node::Block(mirs) => {
                let mut result = None;

                for mir in mirs {
                    result = self.compile_mir(mir);
                }

                result
            },

            Node::Call(name, target_mir, arg_mirs) => {
                let mut args = Vec::with_capacity(arg_mirs.len() + 1);

                let target_ref = self.compile_mir(target_mir);
                args.push(self.coalesce_none(target_ref));

                for arg_mir in arg_mirs {
                    let value_ref = self.compile_mir(arg_mir);
                    args.push(self.coalesce_none(value_ref));
                }

                let (args_array_ref, arg_count) = self.build_args_array_alloca_store(args);

                // Call the "call" host fn with the arguments as an array:
                // `1 + 2` => `call('+', [1, 2], 2)`
                let mut args = [
                    self.build_str_global_const_ref(name),
                    args_array_ref,
                    self.const_u64(arg_count)
                ];
                let call_fn = &self.c.host_fns["call"];
                let result_ref = self.build_call(call_fn.type_ref, call_fn.func_ref, &mut args);

                Some(result_ref)
            },

            Node::CreateClosure(mir_func_ref, captures) => {
                // TODO: Infer the function name from the assignment
                let func_name = self.c.func_name_gen.next_string("fn");
                let func = &self.mir_module.functions[mir_func_ref.i];

                let compiled = FunctionCompiler::compile(
                    self.c,
                    &self.mir_module,
                    func,
                    &func_name,
                    true,
                    true
                );

                match compiled.closure_t {
                    None => {
                        let result = self.build_ptr_any(AnyT::FunctionPtr, compiled.trampoline_ref.unwrap());

                        Some(result)
                    },
                    Some(closure_t) => {
                        let closure_ptr = self.build_malloc(LLVMSizeOf(closure_t));

                        let ptr = self.build_gep(closure_t, closure_ptr, &mut [
                            self.const_u64(0)
                        ]);

                        LLVMBuildStore(self.builder, compiled.trampoline_ref.unwrap(), ptr);

                        for (i, capture) in captures.iter().enumerate() {
                            let captured_value_ref = match capture {
                                CaptureFrom::Capture(capture_ref) => self.build_capture_load(capture_ref),
                                CaptureFrom::Param(param_ref) => self.build_param_load(param_ref),
                                CaptureFrom::Local(local_ref) => self.build_local_load(local_ref)
                            };

                            let ptr = self.build_gep(closure_t, closure_ptr, &mut [
                                self.const_i32(0),
                                self.const_i32((i + 1) as i32),
                            ]);
                            LLVMBuildStore(self.builder, captured_value_ref, ptr);
                        }

                        let result = self.build_ptr_any(AnyT::Closure, closure_ptr);

                        Some(result)
                    }
                }
            },

            Node::If(_, _, _) => todo!("Support If")
        }
    }

    unsafe fn compile_fn_trampoline(
        &mut self,
        fn_name: &str,
        compiled_fn_type: LLVMTypeRef,
        compiled_ref: LLVMValueRef,
        has_capture_struct: bool,
        internal: bool
    ) -> LLVMValueRef {
        let c_name = CString::new(format!("{}_trampoline", fn_name)).unwrap();
        let mut args = Vec::with_capacity(3);
        args.push(LLVMPointerTypeInContext(self.c.context, 0 as c_uint)); // args: *const Value

        // TODO: Verify the arg count before calling
        // LLVMInt64TypeInContext(self.context)                              // arg_count: u64

        if has_capture_struct {
            args.push(LLVMPointerTypeInContext(self.c.context, 0 as c_uint)); // captures: *const <capture struct>
        }

        let trampoline_t = LLVMFunctionType(self.c.any_t, args.as_mut_ptr(), args.len() as u32, 0);
        let trampoline_fn_ref = LLVMAddFunction(self.c.module, c_name.as_ptr(), trampoline_t);

        if internal {
            LLVMSetLinkage(trampoline_fn_ref, LLVMLinkage::LLVMInternalLinkage);
        }

        let block = LLVMAppendBasicBlockInContext(self.c.context, trampoline_fn_ref, c_str!("entry"));

        LLVMPositionBuilderAtEnd(self.builder, block);

        let array_args_ref = LLVMGetParam(trampoline_fn_ref, 0);

        let mut args = Vec::with_capacity(self.func.param_count + 1);
        for i in 0..self.func.param_count {
            let ptr = self.build_gep(self.c.any_t, array_args_ref, &mut [
                self.const_u64(i as u64)
            ]);

            let name = CString::new(format!("arg.{}", i)).unwrap();

            args.push(LLVMBuildLoad2(self.builder, self.c.any_t, ptr, name.as_ptr()));
        }

        if has_capture_struct {
            args.push(LLVMGetParam(trampoline_fn_ref, 1));
        }

        let result = LLVMBuildCall2(
            self.builder,
            compiled_fn_type,
            compiled_ref,
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!("result")
        );

        LLVMBuildRet(self.builder, result);

        trampoline_fn_ref
    }

    unsafe fn closure_struct_type(c: &CompilerModuleContext, capture_count: usize) -> LLVMTypeRef {
        let mut fields = Vec::with_capacity(1 + capture_count);

        // Function pointer
        fields.push(LLVMPointerTypeInContext(c.context, 0));

        for _ in 0..capture_count {
            fields.push(c.any_t);
        }

        LLVMStructTypeInContext(c.context, fields.as_mut_ptr(), fields.len() as u32, 0)
    }

    unsafe fn build_const_any(&mut self, value: Any) -> LLVMValueRef {
        let (t, v) = value.into_raw();

        LLVMConstNamedStruct(self.c.any_t, [
            LLVMConstInt(LLVMInt32TypeInContext(self.c.context), t as c_ulonglong, 0),
            LLVMConstInt(LLVMInt64TypeInContext(self.c.context), v as c_ulonglong, 0),
        ].as_mut_ptr(), 2)
    }

    unsafe fn build_ptr_any(&mut self, typ: AnyT, ptr_value_ref: LLVMValueRef) -> LLVMValueRef {
        let name = self.stmt_name_gen.next("ptr_to_int");
        let ptr_value_int_ref = LLVMBuildPtrToInt(
            self.builder,
            ptr_value_ref,
            LLVMInt64TypeInContext(self.c.context), name.as_ptr()
        );

        let value_ref = LLVMConstNamedStruct(self.c.any_t, [
            self.const_i32(typ.into_raw()),
            LLVMGetPoison(LLVMInt64TypeInContext(self.c.context))
        ].as_mut_ptr(), 2);

        let name = self.stmt_name_gen.next("value");
        LLVMBuildInsertValue(self.builder, value_ref, ptr_value_int_ref, 1, name.as_ptr())
    }

    unsafe fn build_param_load(&mut self, param_ref: &ParamRef) -> LLVMValueRef {
        LLVMGetParam(self.func_ref, param_ref.i as c_uint)
    }

    unsafe fn build_local_store(&mut self, local_ref: &StackFrameLocalRef, value_ref: LLVMValueRef) {
        self.local_refs[local_ref.i] = Some(value_ref);
    }

    unsafe fn build_local_load(&mut self, local_ref: &StackFrameLocalRef) -> LLVMValueRef {
        self.local_refs[local_ref.i].expect("Local get before set")
    }

    unsafe fn build_capture_load(&mut self, capture_ref: &CaptureRef) -> LLVMValueRef {
        let mut indices = [self.const_i32(0), self.const_i32(capture_ref.i as i32 + 1)];
        let ptr = self.build_gep(
            self.capture_struct_type.unwrap(),
            LLVMGetParam(self.func_ref, self.capture_arg_index.unwrap() as c_uint),
            &mut indices // The +1 is because the first element of the struct is the function pointer
        );

        let name = self.stmt_name_gen.next("capture_get");
        LLVMBuildLoad2(self.builder, self.c.any_t, ptr, name.as_ptr())
    }

    unsafe fn build_gep(&mut self, type_ref: LLVMTypeRef, ptr_ref: LLVMValueRef, indices: &mut [LLVMValueRef]) -> LLVMValueRef {
        let name = self.stmt_name_gen.next("struct_ptr");

        LLVMBuildGEP2(self.builder, type_ref, ptr_ref, indices.as_mut_ptr(), indices.len() as u32, name.as_ptr())
    }

    unsafe fn build_str_global_const_ref(&mut self, str: &str) -> LLVMValueRef {
        let name_ref_name = self.c.str_const_name_gen.next("str");
        let c_name = CString::new(str.as_bytes()).unwrap();

        LLVMBuildGlobalStringPtr(self.builder, c_name.as_ptr(), name_ref_name.as_ptr())
    }

    unsafe fn build_args_array_alloca_store(&mut self, args: Vec<LLVMValueRef>) -> (LLVMValueRef, u64) {
        let arg_array_type = LLVMArrayType2(self.c.any_t, args.len() as u64);

        let args_array_ref_name = self.stmt_name_gen.next("args_array");
        let args_array_ref = LLVMBuildAlloca(self.builder, arg_array_type, args_array_ref_name.as_ptr());

        let arg_count = args.len();

        for (i, arg_ref) in args.into_iter().enumerate() {
            let n = self.stmt_name_gen.next("arg");
            let ptr_to_args_array_element = LLVMBuildGEP2(
                self.builder,
                arg_array_type,
                args_array_ref,
                [
                    self.const_u64(0),
                    self.const_u64(i as u64)
                ].as_mut_ptr(),
                2,
                n.as_ptr()
            );
            LLVMBuildStore(self.builder, arg_ref, ptr_to_args_array_element);
        }

        (args_array_ref, arg_count as u64)
    }

    // unsafe fn build_args_array_insertvalue(&mut self, args: Vec<LLVMValueRef>) -> (LLVMValueRef, u64) {
    //     let mut poison_array_vals = Vec::with_capacity(args.len());
    //     for _ in 0..args.len() {
    //         poison_array_vals.push(LLVMGetPoison(self.c.any_t));
    //     }
    //
    //     let array_ref = LLVMConstArray2(self.c.any_t, poison_array_vals.as_mut_ptr(), poison_array_vals.len() as u64);
    //     let mut result = array_ref;
    //
    //     let arg_count = args.len();
    //
    //     for (i, arg) in args.into_iter().enumerate() {
    //         let name = self.stmt_name_gen.next("args_array");
    //         result = LLVMBuildInsertValue(self.builder, array_ref, arg, i as c_uint, name.as_ptr());
    //     }
    //
    //     (array_ref, arg_count as u64)
    // }

    unsafe fn build_call(
        &mut self,
        func_type: LLVMTypeRef,
        func_ref: LLVMValueRef,
        args: &mut [LLVMValueRef]
    ) -> LLVMValueRef {
        let name = self.stmt_name_gen.next("result");

        LLVMBuildCall2(
            self.builder,
            func_type,
            func_ref,
            args.as_mut_ptr(),
            args.len() as c_uint,
            name.as_ptr()
        )
    }

    // TODO: This leaks memory
    unsafe fn build_malloc(&mut self, size: LLVMValueRef) -> LLVMValueRef {
        let malloc = &self.c.host_fns["mallocc"];
        let name = self.stmt_name_gen.next("malloc");

        LLVMBuildCall2(self.builder, malloc.type_ref, malloc.func_ref, [size].as_mut_ptr(), 1, name.as_ptr())
    }

    unsafe fn coalesce_none(&mut self, value_ref: Option<LLVMValueRef>) -> LLVMValueRef {
        match value_ref {
            None => self.build_const_any(Any::none()),
            Some(value_ref) => value_ref
        }
    }

    unsafe fn const_u64(&self, value: u64) -> LLVMValueRef {
        LLVMConstInt(LLVMInt64TypeInContext(self.c.context), value, 0)
    }

    unsafe fn const_i32(&self, value: i32) -> LLVMValueRef {
        // TODO: Should this be a transmute? Test with negative numbers
        LLVMConstInt(LLVMInt32TypeInContext(self.c.context), value as u64, 1)
    }
}