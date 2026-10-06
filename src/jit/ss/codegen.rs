use crate::lir::*;

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Module, ModuleError, Linkage};
use cranelift_codegen::{Context, ir, isa};


pub struct Codegen {
    func_ctx: FunctionBuilderContext,
    ctx: Option<Context>,

    module: JITModule,
    lir_prog: LirProgram
}

impl Codegen {
    pub fn new(lir_prog: LirProgram) -> Result<Self, ModuleError> {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        let module = JITModule::new(builder);

        Ok(Self {
            func_ctx: FunctionBuilderContext::new(),
            ctx: None,
            module,
            lir_prog
        })
    }

    fn lirtype_to_clif_type(&self, lt: LirType) -> ir::Type {
        match lt {
            LirType::I64 => ir::types::I64,
            LirType::F64 => ir::types::F64,
            _ => unreachable!()
        }
    }

    fn make_cranelift_sig(&self, params: &Vec<LirType>, returns: &LirType) -> ir::Signature {
        let mut sig = self.module.make_signature();
        
        for param in params {
            sig.params.push(ir::AbiParam::new(self.lirtype_to_clif_type(*param)));
        } 
        
        sig.returns.push(ir::AbiParam::new(self.lirtype_to_clif_type(*returns)));
        sig
    }

    pub fn start_function(&mut self, func_id: LirFunctionId) -> &Context {
        let lir_fn = self
            .lir_prog
            .functions
            .get(func_id.0 as usize)
            .unwrap();

        let sig = self.make_cranelift_sig(
            &lir_fn.params, 
            &lir_fn.returns
        ); // Create the signature

        // Define the function
        let func = ir::Function::with_name_signature(
            ir::UserFuncName::user(0, func_id.0),
            sig
        );
        
        let mut ctx = self.module.make_context();
        ctx.func = func;

        self.ctx = Some(ctx);
        &self.ctx
    }

    pub fn compile_function(&mut self, func_id: LirFunctionId) -> *const u8 {
        
    }
}