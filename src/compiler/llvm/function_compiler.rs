use std::ffi::{c_uint, CString};
use llvm_sys::prelude::*;
use llvm_sys::core::*;
use llvm_sys::LLVMLinkage;
use crate::compiler::llvm::c_str;
use crate::compiler::llvm::compiler_module_context::CompilerModuleContext;
use crate::compiler::llvm::symbol_name_counter::SymbolNameCounter;
use crate::lir;

pub struct FunctionCompiler<'a> {
    local_types: Vec<LLVMTypeRef>,
    param_types: Vec<LLVMTypeRef>,

    local_refs: Vec<Option<LLVMValueRef>>,

    stmt_name_gen: SymbolNameCounter,

    c: &'a mut CompilerModuleContext,

    lir_module: &'a lir::Module,
    func: &'a lir::Function,
    func_ref: LLVMValueRef
}

pub struct FunctionCompileResult {
    pub func_type_ref: LLVMTypeRef,
    pub func_ref: LLVMValueRef
}

impl <'a> FunctionCompiler<'a> {
    pub unsafe fn compile(
        c: &'a mut CompilerModuleContext,
        lir_module: &'a lir::Module,
        func: &'a lir::Function,
        name: &str,
        exported: bool
    ) -> FunctionCompileResult {
        let mut param_types = Vec::with_capacity(func.param_types.len());
        for param_type in &func.param_types {
            param_types.push(c.llvm_type_of(*param_type));
        }

        let llvm_return_type = c.llvm_type_of(func.return_type);

        let func_type_ref = LLVMFunctionType(
            llvm_return_type,
            param_types.as_mut_ptr(),
            param_types.len() as c_uint,
            0
        );

        let fn_name = CString::new(name).unwrap();
        let func_ref = LLVMAddFunction(c.module, fn_name.as_ptr(), func_type_ref);

        if !exported {
            LLVMSetLinkage(func_ref, LLVMLinkage::LLVMInternalLinkage);
        }

        let mut local_types = Vec::with_capacity(func.local_types.len());
        for local_type in &func.local_types {
            local_types.push(c.llvm_type_of(*local_type));
        }

        let mut local_refs = Vec::new();
        local_refs.resize(func.local_types.len(), None);

        let mut function_builder = FunctionCompiler {
            local_types,
            param_types,

            local_refs,

            stmt_name_gen: SymbolNameCounter::new(),

            c,

            lir_module,
            func,
            func_ref
        };

        function_builder.compile_basic_block(&func.entry, "entry");

        FunctionCompileResult { func_type_ref, func_ref }
    }

    unsafe fn compile_basic_block(&mut self, lir_basic_block: &lir::BasicBlock, name: &str) -> LLVMBasicBlockRef {
        let name = CString::new(name).unwrap();
        let llvm_basic_block = LLVMAppendBasicBlockInContext(self.c.context, self.func_ref, name.as_ptr());
        let builder = LLVMCreateBuilderInContext(self.c.context);
        LLVMPositionBuilderAtEnd(builder, llvm_basic_block);

        for instruction in &lir_basic_block.code {
            match instruction {
                lir::Instruction::LocalSet(local_ref, value_ref, type_ref) => {
                    let value_ref = self.llvm_value_ref_of(*value_ref);

                    self.local_refs[local_ref.i] = Some(value_ref);
                }

                lir::Instruction::CompileTimeSet(_, _, _) => panic!("Cannot compile CompileTimeSet"),

                lir::Instruction::CallIntrinsicFunction(_, _, _, _) => {}

                lir::Instruction::Return(value_ref, _) => {
                    let value_ref = self.llvm_value_ref_of(*value_ref);

                    LLVMBuildRet(builder, value_ref);
                }

                lir::Instruction::If(_, _, _, _) => {}
            }
        }

        LLVMDisposeBuilder(builder);

        llvm_basic_block
    }

    unsafe fn llvm_value_ref_of(&mut self, lir_value_ref: lir::ValueRef) -> LLVMValueRef {
        match lir_value_ref {
            lir::ValueRef::None => self.const_u8(0),
            lir::ValueRef::Bool(value) => self.const_u8(if value { 1 } else { 0 }),
            lir::ValueRef::Int(value) => self.const_i64(value),
            lir::ValueRef::Float(_) => todo!("Support float consts"),
            lir::ValueRef::ComptimeExport(_) => todo!("Support comptime exports"),
            lir::ValueRef::Param(param_ref) => LLVMGetParam(self.func_ref, param_ref.i as c_uint),
            lir::ValueRef::Local(local_ref) => self.local_refs[local_ref.i].expect("Local get before set")
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