use crate::lir::*;

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Module, ModuleError};
use cranelift_codegen::{Context, ir, isa};


pub struct Codegen {
    builder_context: FunctionBuilderContext,
    context: codegen::Context,
    module: JITModule,
    lir_prog: LirProgram
}

impl Codegen {
    pub fn new(lir_prog: LirProgram) -> Result<Self, ModuleError> {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        let module = JITModule::new(builder);

        Ok(Self {
            builder_context: FunctionBuilderContext::new(),
            context: module.make_context(),
            module,
            lir_prog
        })
    }

    fn lirtype_to_abi_param(&self, lt: LirType) -> ir::AbiParam {
        match lt {
            LirType::I64 => ir::AbiParam::new(ir::types::I64),
            LirType::F64 => ir::AbiParam::new(ir::types::F64),
            _ => unreachable!()
        }
    }

    fn make_cranelift_sig(&self, params: &Vec<LirType>, returns: &LirType) -> ir::Signature {
        let mut sig = ir::Signature::new(self.module.isa().default_call_conv());
        
        for param in params {
            sig.params.push(self.lirtype_to_abi_param(*param));
        } 
        
        sig.returns.push(self.lirtype_to_abi_param(*returns));
        sig
    }

    pub fn start_function(&mut self, func_id: LirFunctionId) -> Context {
        let lir_fn = self.lir_prog.functions.get(func_id.0 as usize).unwrap();

        let mut func = ir::Function::with_name_signature(
            ir::UserFuncName::user(0, func_id.0),
            self.make_cranelift_sig(&lir_fn.params, &lir_fn.returns)
        );
        
        let ctx = Context::for_function(func);
        ctx
    }
}