use std::ffi::CString;
use std::os::raw::c_char;
use binaryen_sys::*;
use lib::{ValueT, ValueV};
use crate::backend::wasm::wasm_compiler::CompileMirResult::{NoResult, Result};
use crate::compiler;
use crate::compiler::mir;
use crate::compiler::mir::Node;

pub struct WasmCompiler<'a> {
    mir_module: &'a compiler::Module,
    module: BinaryenModuleRef,
    any_type: BinaryenType,

    fn_name_i: u32
}

impl <'a> Drop for WasmCompiler<'a> {
    fn drop(&mut self) {
        unsafe { BinaryenModuleDispose(self.module) }
    }
}

impl <'a> WasmCompiler<'a> {
    pub fn new(mir_module: &'a compiler::Module) -> Self {
        let module = unsafe {
            let module = BinaryenModuleCreate();

            BinaryenModuleSetFeatures(module, BinaryenFeatureMultivalue());

            module
        };

        let any_type = unsafe {
            let mut tuple_type = [BinaryenTypeInt32(), BinaryenTypeInt64()];

            BinaryenTypeCreate(tuple_type.as_mut_ptr(), tuple_type.len() as u32)
        };

        Self {
            mir_module,
            module,
            any_type,

            fn_name_i: 1
        }
    }

    pub fn compile(&mut self) -> Vec<u8> {
        let mut output = Vec::<u8>::with_capacity(1 * 1024 * 1024);

        unsafe {
            let main_fn_name = String::from("main");

            // TODO: Compile the other functions as well
            self.compile_fn(&self.mir_module.runtime_main, Some(main_fn_name.as_str()));

            // BinaryenModuleAutoDrop(self.module);

            let fn_c_name = CString::new(main_fn_name).unwrap();
            BinaryenAddFunctionExport(self.module, fn_c_name.as_ptr(), fn_c_name.as_ptr());

            let is_valid = BinaryenModuleValidate(self.module);
            println!("Module valid: {}", is_valid);

            println!("----------- Module before optimize");
            BinaryenModulePrint(self.module);

            // BinaryenModuleOptimize(self.module);
            // println!("----------- Module after optimize");
            // BinaryenModulePrint(self.module);

            let size = BinaryenModuleWrite(self.module, output.as_mut_ptr() as *mut c_char, output.capacity());
            if size == output.capacity() {
                todo!("Not enough space in the buffer to save wasm bytecode");
            }
            output.set_len(size);

            output
        }
    }

    unsafe fn compile_fn(&mut self, function: &mir::Function, name: Option<&str>) -> BinaryenFunctionRef {
        // TODO: Better names
        let name = match name {
            None => CString::new(format!("fn_{}", self.fn_name_i)).unwrap(),
            Some(name) => CString::new(name).unwrap()
        };
        self.fn_name_i += 1;

        let mut param_types = Vec::with_capacity(function.param_count);
        for _ in 0..function.param_count {
            param_types.push(BinaryenTypeInt32());
            param_types.push(BinaryenTypeInt64());
        }

        let mut local_types = Vec::with_capacity(function.local_count);
        for _ in 0..function.local_count {
            local_types.push(self.any_type);
        }

        let params = BinaryenTypeCreate(param_types.as_mut_ptr(), param_types.len() as u32);

        let body = match self.compile_mir(&function.body) {
            Result(expr) => expr,
            NoResult(_) => panic!("This shouldn't happen")
        };

        BinaryenAddFunction(
            self.module,
            name.as_ptr(),
            params,
            self.any_type,
            local_types.as_mut_ptr(),
            local_types.len() as u32,
            body
        )
    }

    unsafe fn compile_mir(&mut self, mir: &mir::MIR) -> CompileMirResult {
        match &mir.node {
            Node::Nop => Result(self.make_const_none_tuple()),

            Node::CompileTimeRef(_) => todo!("Support CompileTimeRef"),
            Node::CompileTimeSet(_, _) => todo!("Support CompileTimeSet"),

            Node::GlobalRef(_) => todo!("Support GlobalRef"),
            Node::ConstStringRef(_) => todo!("Support ConstStringRef"),

            Node::LiteralBool(value) => {
                let (t, v) = self.make_const_any(ValueT::bool(*value));

                Result(self.make_tuple([t, v]))
            }

            Node::LiteralI64(value) => {
                let (t, v) = self.make_const_any(ValueT::i64(*value));

                Result(self.make_tuple([t, v]))
            },

            Node::LiteralF64(value) => {
                let (t, v) = self.make_const_any(ValueT::f64(*value));

                Result(self.make_tuple([t, v]))
            },

            Node::CaptureRef(_) => todo!("Support CaptureRef"),
            Node::ParamRef(_) => todo!("Support ParamRef"),

            Node::LocalSet(local_ref, mir) => {
                NoResult(match self.compile_mir(mir) {
                    Result(tuple) => BinaryenLocalSet(self.module, local_ref.i as u32, tuple),
                    NoResult(expr) => self.make_block(&mut [
                        expr,
                        BinaryenLocalSet(self.module, local_ref.i as u32, self.make_const_none_tuple()),
                    ], false)
                })
            },

            Node::LocalGet(local_ref) => {
                // TODO: This doesn't support getting parameters
                // if local_ref.i < arg_locals

                Result(BinaryenLocalGet(self.module, local_ref.i as u32, self.any_type))
            },

            Node::Block(mirs) => {
                let mut exprs = Vec::with_capacity(mirs.len());
                for (i, mir) in mirs.iter().enumerate() {
                    let expr = self.compile_mir(mir);

                    if i == mirs.len() - 1 {
                        match expr {
                            Result(a) => exprs.push(a),
                            NoResult(expr) => {
                                exprs.push(expr);
                                exprs.push(self.make_const_none_tuple());
                            }
                        }
                    } else {
                        match expr {
                            Result(a) => exprs.push(BinaryenDrop(self.module, a)),
                            NoResult(expr) => exprs.push(expr)
                        }
                    }
                }

                Result(self.make_block(exprs.as_mut_slice(), true))
            },

            Node::Call(_, _, _) => todo!("Support Call"),

            Node::CreateClosure(_, _) => todo!("Support CreateClosure"),

            Node::If(_, _, _) => todo!("Support If"),
        }
    }

    unsafe fn make_tuple(&self, mut components: [BinaryenExpressionRef; 2]) -> BinaryenExpressionRef {
        BinaryenTupleMake(self.module, components.as_mut_ptr(), components.len() as u32)
    }

    unsafe fn make_block(&self, exprs: &mut [BinaryenExpressionRef], returns_value: bool) -> BinaryenExpressionRef {
        BinaryenBlock(
            self.module,
            std::ptr::null(),
            exprs.as_mut_ptr(),
            exprs.len() as u32,
            if returns_value { BinaryenTypeAuto() } else { BinaryenTypeNone() }
        )
    }

    unsafe fn make_const_any(&self, value: (ValueT, ValueV)) -> (BinaryenExpressionRef, BinaryenExpressionRef) {
        let t = BinaryenConst(self.module, BinaryenLiteralInt32(value.0.to_literal()));
        let v = BinaryenConst(self.module, BinaryenLiteralInt64(value.1.to_literal()));

        (t, v)
    }

    unsafe fn make_const_none_tuple(&self) -> BinaryenExpressionRef {
        let (t, v) = self.make_const_any(ValueT::none());

        self.make_tuple([t, v])
    }
}

enum CompileMirResult {
    Result(BinaryenExpressionRef),
    NoResult(BinaryenExpressionRef)
}