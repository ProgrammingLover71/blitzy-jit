use std::mem;

use cranelift::codegen::ir::{self, AbiParam, Function, InstBuilder, MemFlags, StackSlotData, StackSlotKind, UserFuncName};
use cranelift::codegen::isa::CallConv;
use cranelift::codegen::Context;
use cranelift::frontend::FunctionBuilder;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use crate::rt::bytecode;

#[no_mangle]
pub extern "C" fn tier1_host_call(arg: i64) -> i64 {
    arg
}

pub struct Tier1Jit {
    module: JITModule,
}

impl Tier1Jit {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);
        Self { module }
    }

    pub fn compile_block(&mut self, block: &bytecode::Block<'_>) -> Result<unsafe extern "C" fn() -> i64, String> {
        let mut context = self.module.make_context();
        let mut func = Function::with_name_signature(
            UserFuncName::new("tier1_entry"),
            Signature::new(CallConv::SystemV),
        );
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

        let reg_slot = builder.func.create_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            (block.used_regs.max(1) * 8) as u32,
            8,
        ));
        let base = builder.ins().stack_addr(types::I64, reg_slot, 0);

        let call_sig = {
            let mut sig = Signature::new(CallConv::SystemV);
            sig.params.push(AbiParam::new(types::I64));
            sig.returns.push(AbiParam::new(types::I64));
            sig
        };
        let host_id = self.module.declare_func("tier1_host_call", Linkage::Import, &call_sig)
            .map_err(|err| err.to_string())?;
        let host_ref = self.module.declare_func_in_func(host_id, &mut builder.func);

        for inst in &block.instructions {
            match inst {
                bytecode::Opcode::Load { dst, val, .. } => {
                    let imm = match val {
                        crate::rt::value::Value::Int(v) => *v as i64,
                        crate::rt::value::Value::Bool(b) => *b as i64,
                        _ => 0,
                    };
                    let ptr = builder.ins().iadd_imm(base, *dst as i32 * 8);
                    builder.ins().store(MemFlags::new(), builder.ins().iconst(types::I64, imm), ptr, 0);
                }
                bytecode::Opcode::Move { dst, src, .. } => {
                    let value = builder.ins().load(
                        types::I64,
                        MemFlags::new(),
                        builder.ins().iadd_imm(base, *src as i32 * 8),
                        0,
                    );
                    let ptr = builder.ins().iadd_imm(base, *dst as i32 * 8);
                    builder.ins().store(MemFlags::new(), value, ptr, 0);
                }
                bytecode::Opcode::Add { dst, arg1, arg2, .. } => {
                    let lhs = builder.ins().load(
                        types::I64,
                        MemFlags::new(),
                        builder.ins().iadd_imm(base, *arg1 as i32 * 8),
                        0,
                    );
                    let rhs = builder.ins().load(
                        types::I64,
                        MemFlags::new(),
                        builder.ins().iadd_imm(base, *arg2 as i32 * 8),
                        0,
                    );
                    let sum = builder.ins().iadd(lhs, rhs);
                    let ptr = builder.ins().iadd_imm(base, *dst as i32 * 8);
                    builder.ins().store(MemFlags::new(), sum, ptr, 0);
                }
                bytecode::Opcode::Call { dst, reg, .. } => {
                    let arg = builder.ins().load(
                        types::I64,
                        MemFlags::new(),
                        builder.ins().iadd_imm(base, *reg as i32 * 8),
                        0,
                    );
                    let result = builder.ins().call(host_ref, &[arg]);
                    let ptr = builder.ins().iadd_imm(base, *dst as i32 * 8);
                    builder.ins().store(MemFlags::new(), result, ptr, 0);
                }
                bytecode::Opcode::Return { reg, .. } => {
                    let value = builder.ins().load(
                        types::I64,
                        MemFlags::new(),
                        builder.ins().iadd_imm(base, *reg as i32 * 8),
                        0,
                    );
                    builder.ins().return_(&[value]);
                }
                _ => {
                    return Err(format!("unsupported opcode in Tier1 JIT: {:?}", inst));
                }
            }
        }

        builder.finalize();
        context.func = func;
        let id = self.module.declare_func("tier1_entry", Linkage::Export, &context.func.signature)
            .map_err(|err| err.to_string())?;
        self.module.define_function(id, &mut context).map_err(|err| err.to_string())?;
        self.module.clear_context(&mut context);
        let entry = self.module.get_finalized_function(id);
        Ok(unsafe { mem::transmute::<*const u8, unsafe extern "C" fn() -> i64>(entry as *const u8) })
    }
}
