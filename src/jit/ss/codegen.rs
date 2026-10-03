use crate::hir::*;

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Module, ModuleError};
use cranelift_codegen::{ir, isa};


pub struct Codegen {
    builder_context: FunctionBuilderContext,
    context: codegen::Context,
    module: JITModule,
}

impl Codegen {
    pub fn new() -> Result<Self, ModuleError> {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        let module = JITModule::new(builder);

        Ok(Self {
            builder_context: FunctionBuilderContext::new(),
            context: module.make_context(),
            module,
        })
    }
}