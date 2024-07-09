use std::ffi::{c_uint, CString};
use llvm_sys::prelude::*;
use llvm_sys::core::*;
use crate::compiler::llvm::compiler_module_context::{CompilerModuleContext, FunctionDeclaration};
use crate::compiler::llvm::symbol_name_counter::SymbolNameCounter;
use crate::lir;
use crate::types::IntrinsicFn;

pub struct FunctionCompiler<'a> {
    decl: &'a FunctionDeclaration,

    local_refs: Vec<Option<LLVMValueRef>>,

    stmt_name_gen: SymbolNameCounter,

    c: &'a mut CompilerModuleContext,

    lir_module: &'a lir::Module
}

impl <'a> FunctionCompiler<'a> {
    pub unsafe fn compile(
        c: &'a mut CompilerModuleContext,
        lir_module: &'a lir::Module,
        func: &'a lir::Function,
        decl: &'a FunctionDeclaration
    ) {
        let mut local_refs = Vec::new();
        local_refs.resize(func.local_types.len(), None);

        let mut function_builder = FunctionCompiler {
            decl,

            local_refs,

            stmt_name_gen: SymbolNameCounter::new(),

            c,

            lir_module
        };

        function_builder.compile_basic_block(&func.entry, "entry");
    }

    unsafe fn compile_basic_block(&mut self, lir_basic_block: &lir::BasicBlock, name: &str) -> LLVMBasicBlockRef {
        let name = CString::new(name).unwrap();
        let llvm_basic_block = LLVMAppendBasicBlockInContext(self.c.context, self.decl.func_ref, name.as_ptr());
        let builder = LLVMCreateBuilderInContext(self.c.context);
        LLVMPositionBuilderAtEnd(builder, llvm_basic_block);

        for instruction in &lir_basic_block.code {
            match instruction {
                lir::Instruction::LocalSet(local_ref, value_ref, _) => {
                    let value_ref = self.llvm_value_ref_of(builder, *value_ref);

                    self.local_refs[local_ref.i] = Some(value_ref);
                }

                lir::Instruction::CompileTimeSet(_, _, _) => panic!("Cannot compile CompileTimeSet"),

                lir::Instruction::CreateDynamicClosure(_, _, _, _) => panic!("Cannot compile CreateDynamicClosure"),

                lir::Instruction::CreateClosure(local_ref, func_ref, value_refs) => {
                    let func_decl = &self.c.function_declarations[func_ref.i];

                    let result_ref = match func_decl.closure_struct_type {
                        None => {
                            // No captures, we just need to generate some value -> doesn't matter which
                            assert!(value_refs.is_empty());

                            self.const_lir_value(&lir::Value::None)
                        }

                        Some(closure_struct_type) => {
                            let captures = self.llvm_value_refs_of(builder, value_refs);

                            self.build_struct(builder, closure_struct_type, captures, "closure")
                        }
                    };

                    self.local_refs[local_ref.i] = Some(result_ref);
                }

                lir::Instruction::CallDynamicFunction(_, _, _) => panic!("Cannot compile dynamic function calls"),

                lir::Instruction::CallIntrinsicFunction(local_ref, intrinsic_fn, arg_refs, _) => {
                    let args = self.llvm_value_refs_of(builder, arg_refs);
                    let name = self.stmt_name_gen.next("result");

                    let result_ref = match intrinsic_fn {
                        IntrinsicFn::AddInt => LLVMBuildAdd(builder, args[0], args[1], name.as_ptr())
                    };

                    self.local_refs[local_ref.i] = Some(result_ref);
                }

                lir::Instruction::CallStaticClosureFunction(local_ref, func_ref, closure_ref, arg_refs, _) => {
                    let mut args = Vec::with_capacity(arg_refs.len() + 1);
                    for lir_value_ref in arg_refs {
                        args.push(self.llvm_value_ref_of(builder, *lir_value_ref));
                    }

                    args.push(self.llvm_value_ref_of(builder, *closure_ref));

                    let func_decl = &self.c.function_declarations[func_ref.i];

                    match func_decl.closure_struct_type {
                        None => panic!("Tried to call a non-closure function with a closure, should have been compiled to CallStaticFunction"),
                        Some(_) => {}
                    }

                    let result_ref = self.build_call(
                        builder,
                        func_decl.type_ref,
                        func_decl.func_ref,
                        &mut args
                    );

                    self.local_refs[local_ref.i] = Some(result_ref);
                }

                lir::Instruction::CallStaticFunction(local_ref, func_ref, arg_refs, _) => {
                    let mut args = self.llvm_value_refs_of(builder, arg_refs);
                    let func_decl = &self.c.function_declarations[func_ref.i];

                    match func_decl.closure_struct_type {
                        None => {},
                        Some(_) => panic!("Tried to call a closure function without a closure, should have been compiled to CallStaticClosureFunction")
                    }

                    let result_ref = self.build_call(
                        builder,
                        func_decl.type_ref,
                        func_decl.func_ref,
                        &mut args
                    );

                    self.local_refs[local_ref.i] = Some(result_ref);
                }

                lir::Instruction::CallPtrFunction(_, _, _, _) => todo!("Support compiling CallPtrFunction"),
                lir::Instruction::CallPtrClosureFunction(_, _, _, _, _) => todo!("Support compiling CallPtrClosureFunction"),

                lir::Instruction::Return(value_ref, _) => {
                    let value_ref = self.llvm_value_ref_of(builder, *value_ref);

                    LLVMBuildRet(builder, value_ref);
                }

                lir::Instruction::If(_, _, _, _) => {}
            }
        }

        LLVMDisposeBuilder(builder);

        llvm_basic_block
    }

    unsafe fn build_struct(
        &mut self,
        builder: LLVMBuilderRef,
        struct_type: LLVMTypeRef,
        values: Vec<LLVMValueRef>,
        name_prefix: &str
    ) -> LLVMValueRef {
        let mut poison_vals = Vec::with_capacity(values.len());
        for i in 0..values.len() {
            poison_vals.push(LLVMGetPoison(LLVMStructGetTypeAtIndex(struct_type, i as c_uint)));
        }

        let mut struct_ref = LLVMConstStructInContext(
            self.c.context,
            poison_vals.as_mut_ptr(),
            poison_vals.len() as c_uint,
            LLVMIsPackedStruct(struct_type)
        );

        // let struct_name = self.stmt_name_gen.next("closure");
        // let mut closure_ref = LLVMBuildAlloca(builder, struct_type, struct_name.as_ptr());
        for (i, capture_ref) in values.into_iter().enumerate() {
            let name = self.stmt_name_gen.next(name_prefix);

            struct_ref = LLVMBuildInsertValue(builder, struct_ref, capture_ref, i as c_uint, name.as_ptr());
        }

        struct_ref
    }

    unsafe fn build_call(
        &mut self,
        builder: LLVMBuilderRef,
        func_type: LLVMTypeRef,
        func_ref: LLVMValueRef,
        args: &mut [LLVMValueRef]
    ) -> LLVMValueRef {
        let name = self.stmt_name_gen.next("call_result");

        LLVMBuildCall2(
            builder,
            func_type,
            func_ref,
            args.as_mut_ptr(),
            args.len() as c_uint,
            name.as_ptr()
        )
    }

    unsafe fn llvm_value_refs_of(&mut self, builder: LLVMBuilderRef, lir_value_refs: &[lir::ValueRef]) -> Vec<LLVMValueRef> {
        let mut result = Vec::with_capacity(lir_value_refs.len());

        for lir_value_ref in lir_value_refs {
            result.push(self.llvm_value_ref_of(builder, *lir_value_ref));
        }

        result
    }

    unsafe fn llvm_value_ref_of(&mut self, builder: LLVMBuilderRef, lir_value_ref: lir::ValueRef) -> LLVMValueRef {
        match lir_value_ref {
            lir::ValueRef::None => self.const_lir_value(&lir::Value::None),
            lir::ValueRef::Bool(value) => self.const_lir_value(&lir::Value::Bool(value)),
            lir::ValueRef::Int(value) => self.const_lir_value(&lir::Value::Int(value)),
            lir::ValueRef::Float(_) => todo!("Support float consts"),
            lir::ValueRef::Global(_) => todo!("Support globals"),
            lir::ValueRef::ComptimeExport(_) => todo!("Support comptime exports"),
            lir::ValueRef::Const(const_ref) => self.const_lir_value(&self.lir_module.constants[const_ref.i]),
            lir::ValueRef::Capture(capture_ref) => {
                // The last argument is the capture struct
                let capture_struct_ref = LLVMGetParam(self.decl.func_ref, (self.decl.param_types.len() - 1) as c_uint);

                // TODO: Use the name of the capture variable
                let name = self.stmt_name_gen.next("capture");

                LLVMBuildExtractValue(builder, capture_struct_ref, capture_ref.i as c_uint, name.as_ptr())
            }
            lir::ValueRef::Param(param_ref) => LLVMGetParam(self.decl.func_ref, param_ref.i as c_uint),
            lir::ValueRef::Local(local_ref) => self.local_refs[local_ref.i].expect("Local get before set")
        }
    }

    unsafe fn const_lir_value(&self, lir_value: &lir::Value) -> LLVMValueRef {
        match lir_value {
            lir::Value::None => self.const_u8(0),
            lir::Value::Bool(value) => self.const_u8(if *value { 1 } else { 0 }),
            lir::Value::Int(value) => self.const_i64(*value),
            lir::Value::Float(_) => todo!("Support float consts"),

            // TODO: Type error instead of panic
            lir::Value::Type(_) => panic!("Cannot export Type to runtime as it's not serializable"),
            lir::Value::Closure(_, _) => todo!("Serialize closure")
        }
    }

    unsafe fn const_u8(&self, value: u8) -> LLVMValueRef {
        LLVMConstInt(LLVMInt8TypeInContext(self.c.context), value as u64, 0)
    }

    unsafe fn const_i64(&self, value: i64) -> LLVMValueRef {
        // TODO: Should this be a transmute? Test with negative numbers
        LLVMConstInt(LLVMInt64TypeInContext(self.c.context), value as u64, 1)
    }

    unsafe fn const_u64(&self, value: u64) -> LLVMValueRef {
        LLVMConstInt(LLVMInt64TypeInContext(self.c.context), value, 0)
    }

    unsafe fn const_i32(&self, value: i32) -> LLVMValueRef {
        // TODO: Should this be a transmute? Test with negative numbers
        LLVMConstInt(LLVMInt32TypeInContext(self.c.context), value as u64, 1)
    }
}