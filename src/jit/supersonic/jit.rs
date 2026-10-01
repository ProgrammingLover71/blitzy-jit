use crate::rt::bytecode;

use cranelift::codegen::ir::UserFuncName;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift::codegen::isa::TargetFrontendConfig;


#[derive(Debug, Clone)]
pub struct CodeObject {
	pub name: UserFuncName,
	pub id: cranelift_module::FuncId,
	pub signature: cranelift::codegen::ir::Signature,
	pub code: Vec<u8>,
}

pub struct SupersonicJIT {
	pub module: JITModule,
	pub target_config: TargetFrontendConfig,
	pub functions: Vec<CodeObject>
}


impl SupersonicJIT {
	pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
		let builder = JITBuilder::new(default_libcall_names())?;
		let module = JITModule::new(builder);
		let target_config = module.target_config();

		Ok(Self {
			module,
			target_config,
			functions: Vec::new(),
		})
	}

	pub fn create_function(&mut self, name: &str, signature: cranelift::codegen::ir::Signature) -> Result<&mut CodeObject, Box<dyn std::error::Error>> {
		let id = self.module.declare_function(name, Linkage::Export, &signature)?;
		let code_object = CodeObject {
			name: UserFuncName::user(0, id.as_u32()),
			id,
			signature,
			code: Vec::new(),
		};
		self.functions.push(code_object);
		Ok(self.functions.last_mut().unwrap())
	}

	pub fn finalize_function(&mut self, code_object: &CodeObject, func_ctx: &mut cranelift::codegen::Context) -> Result<(), Box<dyn std::error::Error>> {
		self.module.define_function(code_object.id, func_ctx)?;
		self.module.clear_context(func_ctx);
		self.module.finalize_definitions()?;
		Ok(())
	}

	pub fn get_function_address(&self, code_object: &CodeObject) -> Result<*const u8, Box<dyn std::error::Error>> {
		let func_ptr = self.module.get_finalized_function(code_object.id);
		Ok(func_ptr)
	}

	
	pub fn compile_bytecode(&mut self, bytecode: &bytecode::Block, func_name: UserFuncName) -> Result<(), Box<dyn std::error::Error>> {
		let function = self.functions.iter().find(|f| f.name == func_name).ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::NotFound, format!("Function not found: {:?}", func_name))
		})?;

		let mut func_ctx = self.module.make_context();
		func_ctx.func.signature = function.signature.clone();
		func_ctx.func.name = function.name.clone();

		let mut builder_ctx = FunctionBuilderContext::new();

		{
			let mut builder = FunctionBuilder::new(&mut func_ctx.func, &mut builder_ctx);
			let entry = builder.create_block();
			builder.switch_to_block(entry);
			builder.seal_block(entry);

			let mut reg_values = vec![builder.ins().iconst(types::I64, 0); bytecode.used_regs as usize];
			let mut stack_values: Vec<cranelift::codegen::ir::Value> = Vec::new();

			let emit_literal = |builder: &mut FunctionBuilder, literal: &bytecode::Value<'_>| -> cranelift::codegen::ir::Value {
				match literal {
					bytecode::Value::None => builder.ins().iconst(types::I64, 0),
					bytecode::Value::Int(value) => builder.ins().iconst(types::I64, *value),
					bytecode::Value::Float(value) => builder.ins().f64const(*value),
					bytecode::Value::Bool(value) => builder.ins().iconst(types::I8, if *value { 1 } else { 0 }),
					_ => builder.ins().iconst(types::I64, 0),
				}
			};

			for instruction in &bytecode.instructions {
				match instruction {
					bytecode::Opcode::Return { reg } => {
						let value = reg_values[*reg as usize];
						builder.ins().return_(&[value]);
					}
					bytecode::Opcode::Call { dst, .. } => {
						reg_values[*dst as usize] = builder.ins().iconst(types::I64, 0);
					}
					bytecode::Opcode::Load { dst, val } => {
						reg_values[*dst as usize] = emit_literal(&mut builder, val);
					}
					bytecode::Opcode::Move { dst, src } => {
						reg_values[*dst as usize] = reg_values[*src as usize];
					}
					bytecode::Opcode::Push { reg } => {
						stack_values.push(reg_values[*reg as usize]);
					}
					bytecode::Opcode::Pushi { imm } => {
						stack_values.push(emit_literal(&mut builder, imm));
					}
					bytecode::Opcode::Pop { reg } => {
						reg_values[*reg as usize] = stack_values.pop().unwrap_or_else(|| builder.ins().iconst(types::I64, 0));
					}
					bytecode::Opcode::Add { dst, arg1, arg2 } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = reg_values[*arg2 as usize];
						reg_values[*dst as usize] = builder.ins().iadd(lhs, rhs);
					}
					bytecode::Opcode::Addi { dst, arg1, imm } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = emit_literal(&mut builder, imm);
						reg_values[*dst as usize] = builder.ins().iadd(lhs, rhs);
					}
					bytecode::Opcode::Sub { dst, arg1, arg2 } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = reg_values[*arg2 as usize];
						reg_values[*dst as usize] = builder.ins().isub(lhs, rhs);
					}
					bytecode::Opcode::Subi { dst, arg1, imm } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = emit_literal(&mut builder, imm);
						reg_values[*dst as usize] = builder.ins().isub(lhs, rhs);
					}
					bytecode::Opcode::Mul { dst, arg1, arg2 } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = reg_values[*arg2 as usize];
						reg_values[*dst as usize] = builder.ins().imul(lhs, rhs);
					}
					bytecode::Opcode::Muli { dst, arg1, imm } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = emit_literal(&mut builder, imm);
						reg_values[*dst as usize] = builder.ins().imul(lhs, rhs);
					}
					bytecode::Opcode::Div { dst, arg1, arg2 } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = reg_values[*arg2 as usize];
						reg_values[*dst as usize] = builder.ins().sdiv(lhs, rhs);
					}
					bytecode::Opcode::Divi { dst, arg1, imm } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = emit_literal(&mut builder, imm);
						reg_values[*dst as usize] = builder.ins().sdiv(lhs, rhs);
					}
					bytecode::Opcode::Eq { dst, arg1, arg2 } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = reg_values[*arg2 as usize];
						let cmp = builder.ins().icmp(IntCC::Equal, lhs, rhs);
						reg_values[*dst as usize] = builder.ins().uextend(types::I64, cmp);
					}
					bytecode::Opcode::Neq { dst, arg1, arg2 } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = reg_values[*arg2 as usize];
						let cmp = builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
						reg_values[*dst as usize] = builder.ins().uextend(types::I64, cmp);
					}
					bytecode::Opcode::Gt { dst, arg1, arg2 } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = reg_values[*arg2 as usize];
						let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
						reg_values[*dst as usize] = builder.ins().uextend(types::I64, cmp);
					}
					bytecode::Opcode::Lt { dst, arg1, arg2 } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = reg_values[*arg2 as usize];
						let cmp = builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
						reg_values[*dst as usize] = builder.ins().uextend(types::I64, cmp);
					}
					bytecode::Opcode::Gte { dst, arg1, arg2 } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = reg_values[*arg2 as usize];
						let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
						reg_values[*dst as usize] = builder.ins().uextend(types::I64, cmp);
					}
					bytecode::Opcode::Lte { dst, arg1, arg2 } => {
						let lhs = reg_values[*arg1 as usize];
						let rhs = reg_values[*arg2 as usize];
						let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
						reg_values[*dst as usize] = builder.ins().uextend(types::I64, cmp);
					}
					bytecode::Opcode::LoadName { dst, .. } => {
						reg_values[*dst as usize] = builder.ins().iconst(types::I64, 0);
					}
					bytecode::Opcode::StoreName { src, .. } => {
						let _ = reg_values[*src as usize];
					}
					bytecode::Opcode::Jmp { .. } => {
						// Control-flow jumps are intentionally left for a later pass.
					}
				}
			}

			builder.finalize(self.target_config);
		}

		Ok(())
	}
}
