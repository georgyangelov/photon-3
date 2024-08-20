use std::ffi::{c_uint, CString};
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use crate::compiler::compiler::FunctionDeclaration;
use crate::compiler::symbol_name_counter::SymbolNameCounter;
use crate::{ir, lir};
use crate::vec_map::VecMap;

pub struct FunctionBuilder<'a> {
    llvm_context: LLVMContextRef,
    llvm_module: LLVMModuleRef,

    decl: &'a FunctionDeclaration,
    func: &'a lir::Function,

    local_refs: Vec<Option<LLVMValueRef>>,

    stmt_name_gen: SymbolNameCounter
}

impl <'a> FunctionBuilder<'a> {
    // TODO: Error handling instead of panics
    pub unsafe fn build(
        llvm_context: LLVMContextRef,
        llvm_module: LLVMModuleRef,

        decl: &FunctionDeclaration,
        func: &lir::Function
    ) {
        let mut local_refs = Vec::new();
        local_refs.resize(func.local_count, None);

        let mut fb = FunctionBuilder {
            llvm_context,
            llvm_module,

            decl,
            func,

            local_refs,

            stmt_name_gen: SymbolNameCounter::new()
        };

        fb.compile();
    }

    unsafe fn compile(&mut self) {
        self.compile_basic_block(&self.func.body, "entry");
    }

    unsafe fn compile_basic_block(&mut self, block: &lir::BasicBlock, name: &str) -> LLVMBasicBlockRef {
        let name = CString::new(name).unwrap();
        let basic_block = LLVMAppendBasicBlockInContext(self.llvm_context, self.decl.func_ref, name.as_ptr());
        let builder = LLVMCreateBuilderInContext(self.llvm_context);
        LLVMPositionBuilderAtEnd(builder, basic_block);

        for instruction in &block.code {
            match instruction {
                lir::Instruction::LocalSet(local_ref, value_ref, _) => {
                    let value_ref = self.llvm_value_ref_of(builder, *value_ref);

                    self.local_refs[local_ref.i] = Some(value_ref);
                }
                lir::Instruction::CallIntrinsic(local_ref, intrinsic_fn, arg_refs) => {
                    let args = self.llvm_value_refs_of(builder, arg_refs);
                    let name = self.stmt_name_gen.next("result");

                    let result_ref = match intrinsic_fn {
                        ir::IntrinsicFn::AddInt => LLVMBuildAdd(builder, args[0], args[1], name.as_ptr())
                    };

                    self.local_refs[local_ref.i] = Some(result_ref);
                }
                lir::Instruction::Return(value_ref) => {
                    let value_ref = self.llvm_value_ref_of(builder, *value_ref);

                    LLVMBuildRet(builder, value_ref);
                }
                lir::Instruction::If(_, _, _, _, _) => todo!("Support compiling ifs")
            }
            }

        LLVMDisposeBuilder(builder);

        basic_block
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
            lir::ValueRef::None => self.const_lir_value(&ir::Value::None),
            lir::ValueRef::Bool(value) => self.const_lir_value(&ir::Value::Bool(value)),
            lir::ValueRef::Int(value) => self.const_lir_value(&ir::Value::Int(value)),
            lir::ValueRef::Float(_) => todo!("Support float consts"),
            // lir::ValueRef::Global(_) => todo!("Support globals"),
            // lir::ValueRef::ComptimeExport(_) => todo!("Support comptime exports"),
            // lir::ValueRef::Const(const_ref) => self.const_lir_value(&self.lir_module.constants[const_ref.i]),
            // lir::ValueRef::Capture(capture_ref) => {
            //     // The last argument is the capture struct
            //     let capture_struct_ref = LLVMGetParam(self.decl.func_ref, (self.decl.param_types.len() - 1) as c_uint);
            //
            //     // TODO: Use the name of the capture variable
            //     let name = self.stmt_name_gen.next("capture");
            //
            //     LLVMBuildExtractValue(builder, capture_struct_ref, capture_ref.i as c_uint, name.as_ptr())
            // }
            lir::ValueRef::Param(param_ref) => LLVMGetParam(self.decl.func_ref, param_ref.i as c_uint),
            lir::ValueRef::Local(local_ref) => self.local_refs[local_ref.i].expect("Local get before set")
        }
    }

    unsafe fn const_lir_value(&self, ir_value: &ir::Value) -> LLVMValueRef {
        match ir_value {
            // TODO: Better `void` type
            ir::Value::None => self.const_u8(0),
            ir::Value::Bool(value) => self.const_u8(if *value { 1 } else { 0 }),
            ir::Value::Int(value) => self.const_i64(*value),
            ir::Value::Float(_) => todo!("Support float consts"),

            // TODO: Type error instead of panic
            ir::Value::Type(_) => panic!("Cannot export Type to runtime as it's not serializable"),
            ir::Value::Closure(_, _) => todo!("Serialize closure")
        }
    }

    unsafe fn const_u8(&self, value: u8) -> LLVMValueRef {
        LLVMConstInt(LLVMInt8TypeInContext(self.llvm_context), value as u64, 0)
    }

    unsafe fn const_i64(&self, value: i64) -> LLVMValueRef {
        // TODO: Should this be a transmute? Test with negative numbers
        LLVMConstInt(LLVMInt64TypeInContext(self.llvm_context), value as u64, 1)
    }

    unsafe fn const_u64(&self, value: u64) -> LLVMValueRef {
        LLVMConstInt(LLVMInt64TypeInContext(self.llvm_context), value, 0)
    }

    unsafe fn const_i32(&self, value: i32) -> LLVMValueRef {
        // TODO: Should this be a transmute? Test with negative numbers
        LLVMConstInt(LLVMInt32TypeInContext(self.llvm_context), value as u64, 1)
    }
}