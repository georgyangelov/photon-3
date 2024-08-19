use std::ffi::CString;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use crate::compiler::compiler::FunctionDeclaration;
use crate::compiler::symbol_name_counter::SymbolNameCounter;
use crate::ir;
use crate::ir::{IntrinsicFn, Value};
use crate::vec_map::VecMap;

pub struct FunctionBuilder<'a> {
    llvm_context: LLVMContextRef,
    llvm_module: LLVMModuleRef,

    decl: &'a FunctionDeclaration,
    func: &'a ir::RFunction,

    local_refs: VecMap<ir::LocalRef, LLVMValueRef>,

    stmt_name_gen: SymbolNameCounter
}

impl <'a> FunctionBuilder<'a> {
    // TODO: Error handling instead of panics
    pub unsafe fn build(
        llvm_context: LLVMContextRef,
        llvm_module: LLVMModuleRef,

        decl: &FunctionDeclaration,
        func: &ir::RFunction
    ) {
        let mut fb = FunctionBuilder {
            llvm_context,
            llvm_module,

            decl,
            func,

            local_refs: VecMap::with_capacity(func.locals.len()),

            stmt_name_gen: SymbolNameCounter::new()
        };

        fb.compile();
    }

    unsafe fn compile(&mut self) {
        self.compile_entry_block("entry", &self.func.body);
    }

    unsafe fn compile_entry_block(&mut self, block_name: &str, ir: &ir::IR) -> LLVMBasicBlockRef {
        let name = CString::new(block_name).unwrap();
        let basic_block = LLVMAppendBasicBlockInContext(self.llvm_context, self.decl.func_ref, name.as_ptr());
        let builder = LLVMCreateBuilderInContext(self.llvm_context);
        LLVMPositionBuilderAtEnd(builder, basic_block);

        let result_ref = self.compile_ir(ir, builder);

        LLVMBuildRet(builder, result_ref);

        LLVMDisposeBuilder(builder);

        basic_block
    }

    unsafe fn compile_ir(&mut self, ir: &ir::IR, builder: LLVMBuilderRef) -> LLVMValueRef {
        match &ir.node {
            ir::Node::Nop => self.const_lir_value(&Value::None),
            ir::Node::Constant(value) => self.const_lir_value(value),
            ir::Node::GlobalRef(_) => todo!("Support global refs"),
            ir::Node::ParamRef(_) => todo!("Support param refs"),
            ir::Node::LocalRef(local_ref) => *self.local_refs.get(local_ref).unwrap(),
            ir::Node::CaptureRef(_) => todo!("Support capture refs"),
            ir::Node::LocalSet(local_ref, ir) => {
                let result = self.compile_ir(ir, builder);

                self.local_refs.insert(*local_ref, result);

                self.const_lir_value(&Value::None)
            }
            ir::Node::Block(irs) => {
                if irs.len() == 0 {
                    todo!("This shouldn't be possible, right?")
                }

                let mut result_ref = None;

                for ir in irs {
                    result_ref = Some(self.compile_ir(ir, builder));
                }

                result_ref.unwrap()
            }
            ir::Node::Comptime(_) => panic!("Cannot compile comptime blocks"),
            ir::Node::DynamicCall(_, _, _) => panic!("Cannot compile dynamic calls"),
            ir::Node::DynamicCreateClosure(_, _) => panic!("Cannot compile dynamic closures"),
            ir::Node::StaticCallIntrinsic(intrinsic, args) => {
                let args = self.compile_values(args, builder);
                let name = self.stmt_name_gen.next("result");

                match intrinsic {
                    IntrinsicFn::AddInt => LLVMBuildAdd(builder, args[0], args[1], name.as_ptr())
                }
            }
            ir::Node::StaticCall(_, _) => todo!("Support calls"),
            ir::Node::If(_, _, _) => todo!("Support ifs")
        }
    }

    unsafe fn compile_values(&mut self, irs: &Vec<ir::IR>, builder: LLVMBuilderRef) -> Vec<LLVMValueRef> {
        let mut values = Vec::with_capacity(irs.len());
        for ir in irs {
            let value = self.compile_ir(ir, builder);

            values.push(value);
        }

        values
    }

    unsafe fn const_lir_value(&self, ir_value: &Value) -> LLVMValueRef {
        match ir_value {
            // TODO: Better `void` type
            Value::None => self.const_u8(0),
            Value::Bool(value) => self.const_u8(if *value { 1 } else { 0 }),
            Value::Int(value) => self.const_i64(*value),
            Value::Float(_) => todo!("Support float consts"),

            // TODO: Type error instead of panic
            Value::Type(_) => panic!("Cannot export Type to runtime as it's not serializable"),
            Value::Closure(_, _) => todo!("Serialize closure")
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