use std::ffi::CString;
use std::os::raw::c_char;
use binaryen_sys::*;
use lib::{ValueT};
use crate::backend::wasm::wasm_compiler::CompileMirResult::{NoResult, Separate, Tuple};
use crate::compiler;
use crate::compiler::mir;
use crate::compiler::mir::Node;

pub struct WasmCompiler<'a> {
    mir_module: &'a compiler::Module,
    module: BinaryenModuleRef,

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

        Self { mir_module, module, fn_name_i: 1 }
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

            BinaryenModuleOptimize(self.module);
            println!("----------- Module after optimize");
            BinaryenModulePrint(self.module);

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
            local_types.push(BinaryenTypeInt32());
            local_types.push(BinaryenTypeInt64());
        }

        let params = BinaryenTypeCreate(param_types.as_mut_ptr(), param_types.len() as u32);

        // TODO: Possibly hoist this creation?
        let mut tuple_args = [BinaryenTypeInt32(), BinaryenTypeInt64()];
        let return_type = BinaryenTypeCreate(tuple_args.as_mut_ptr(), tuple_args.len() as u32);

        let body = match self.compile_mir(&function.body) {
            Separate((t, v)) => self.make_tuple([t, v]),
            Tuple(tuple) => tuple,
            NoResult(_) => todo!()
        };

        BinaryenAddFunction(
            self.module,
            name.as_ptr(),
            params,
            return_type,
            local_types.as_mut_ptr(),
            local_types.len() as u32,
            body
        )
    }

    unsafe fn compile_mir(&mut self, mir: &mir::MIR) -> CompileMirResult {
        match &mir.node {
            Node::Nop => Separate(self.make_none_const()),

            Node::CompileTimeRef(_) => todo!("Support CompileTimeRef"),
            Node::CompileTimeSet(_, _) => todo!("Support CompileTimeSet"),

            Node::GlobalRef(_) => todo!("Support GlobalRef"),
            Node::ConstStringRef(_) => todo!("Support ConstStringRef"),

            Node::LiteralBool(value) => {
                let (t, v) = ValueT::bool(*value);
                let t = BinaryenConst(self.module, BinaryenLiteralInt32(t.to_literal()));
                let v = BinaryenConst(self.module, BinaryenLiteralInt64(v.to_literal()));

                Separate((t, v))
            }

            Node::LiteralI64(value) => {
                let (t, v) = ValueT::i64(*value);
                let t = BinaryenConst(self.module, BinaryenLiteralInt32(t.to_literal()));
                let v = BinaryenConst(self.module, BinaryenLiteralInt64(v.to_literal()));

                Separate((t, v))
            },
            Node::LiteralF64(value) => {
                let (t, v) = ValueT::f64(*value);
                let t = BinaryenConst(self.module, BinaryenLiteralInt32(t.to_literal()));
                let v = BinaryenConst(self.module, BinaryenLiteralInt64(v.to_literal()));

                Separate((t, v))
            },

            // TODO: Optimize this -> currently creates consts twice (?)
            Node::LocalSet(local_ref, mir) => {
                NoResult(match self.compile_mir(mir) {
                    Separate((t, v)) => self.make_block(&mut [
                        BinaryenLocalSet(self.module, (local_ref.i * 2) as u32, t),
                        BinaryenLocalSet(self.module, (local_ref.i * 2 + 1) as u32, v)
                    ], false),
                    Tuple(_) => todo!("Use scratch locals for this"),
                    NoResult(_) => panic!("Should not happen - cannot assign NoResult to variable")
                })

                // let (t, v) = self.compile_mir(mir);
                //
                // self.make_block(&mut [
                //     BinaryenLocalSet(self.module, (local_ref.i * 2) as u32, t),
                //     BinaryenLocalSet(self.module, (local_ref.i * 2 + 1) as u32, v),
                //
                //     // TODO: Can we optimize this?
                //     self.make_tuple()
                // ], true)
            },

            Node::LocalGet(local_ref) => {
                Separate((
                    BinaryenLocalGet(self.module, (local_ref.i * 2) as u32, BinaryenTypeInt32()),
                    BinaryenLocalGet(self.module, (local_ref.i * 2 + 1) as u32, BinaryenTypeInt64())
                ))
            },

            Node::Block(mirs) => {
                println!("Block: {:?}", mirs);

                let mut exprs = Vec::with_capacity(mirs.len());
                for (i, mir) in mirs.iter().enumerate() {
                    let expr = self.compile_mir(mir);

                    if i == mirs.len() - 1 {
                        match expr {
                            Separate((a, b)) => exprs.push(self.make_tuple([a, b])),
                            Tuple(a) => exprs.push(a),
                            NoResult(expr) => todo!()
                        }
                    } else {
                        match expr {
                            Separate((a, b)) => {
                                exprs.push(BinaryenDrop(self.module, a));
                                exprs.push(BinaryenDrop(self.module, b));
                            }
                            Tuple(a) => {exprs.push(BinaryenDrop(self.module, a));}
                            NoResult(expr) => { exprs.push(expr); }
                        }
                    }
                }

                Tuple(self.make_block(exprs.as_mut_slice(), true))
            },

            Node::Call(_, _, _) => todo!("Support Call"),

            Node::CreateClosure(_, _) => todo!("Support CreateClosure"),

            Node::If(_, _, _) => todo!("Support If"),
        }
    }

    unsafe fn make_tuple(&self, mut components: [BinaryenExpressionRef; 2]) -> BinaryenExpressionRef {
        BinaryenTupleMake(self.module, components.as_mut_ptr(), components.len() as u32)
    }

    unsafe fn make_block(&self, mut exprs: &mut [BinaryenExpressionRef], returns_value: bool) -> BinaryenExpressionRef {
        BinaryenBlock(
            self.module,
            std::ptr::null(),
            exprs.as_mut_ptr(),
            exprs.len() as u32,
            if returns_value { BinaryenTypeAuto() } else { BinaryenTypeNone() }
        )
    }

    unsafe fn make_none_const(&self) -> (BinaryenExpressionRef, BinaryenExpressionRef) {
        let (t, v) = ValueT::none();
        let t = BinaryenConst(self.module, BinaryenLiteralInt32(t.to_literal()));
        let v = BinaryenConst(self.module, BinaryenLiteralInt64(v.to_literal()));

        (t, v)
    }
}

enum CompileMirResult {
    Separate((BinaryenExpressionRef, BinaryenExpressionRef)),
    Tuple(BinaryenExpressionRef),
    NoResult(BinaryenExpressionRef)
}