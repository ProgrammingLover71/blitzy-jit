use std::collections::HashMap;

use crate::sil::block;
use crate::sil::func;
use crate::sil::inst;
use crate::sil::types;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptId {
    None,
    Id(usize)
}

impl OptId {
    pub fn unwrap(&self) -> usize {
        match self {
            Self::None => panic!("Cannot unwrap OptId::None"),
            Self::Id(i) => *i
        }
    }
}

pub struct ILBuilder<'a> {
    functions: Vec<func::Function<'a>>,
    current_function_idx: OptId,
    current_block_idx: OptId,
    current_var_id: u32
}

impl<'a> ILBuilder<'a> {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            current_function_idx: OptId::None,
            current_block_idx: OptId::None,
            current_var_id: 0,
        }
    }

    pub fn define_function(&mut self, name: String, params: Vec<types::Type<'a>>, ret: types::Type<'a>) -> OptId {
        let func = func::Function {
            name, params, returns: ret,
            blocks: Vec::new(),
            type_map: HashMap::new()
        };
        self.functions.push(func);
        OptId::Id(self.functions.len() - 1)
    }

    pub fn set_current_function(&mut self, function_id: OptId) {
        self.current_function_idx = function_id;
    }

    pub fn get_function(&self, function_id: OptId) -> &func::Function {
        self.functions.get(function_id.unwrap()).unwrap()
    }

    pub fn current_function(&self) -> Option<&func::Function> {
        self.functions.get(self.current_function_idx.unwrap())
    }

    pub fn make_entry_block(&mut self, function_id: OptId) -> OptId {
        let raw_params = self.functions.get(function_id.unwrap()).unwrap().params.clone();
        let params = self.assign_params_to_locations(&raw_params);
        let func = self.functions.get_mut(function_id.unwrap()).unwrap();
                
        func.blocks.push(block::Block {
            insts: Vec::new(),
            params,
            term: None
        });

        OptId::Id(self.functions.len() - 1)
    }

    fn assign_params_to_locations(&mut self, params: &Vec<types::Type<'a>>) -> Vec<(u32, types::Type<'a>)> {
        let mut located_params = Vec::new();

        for type_param in params {
            located_params.push((self.current_var_id, *type_param));
            self.current_var_id += 1;
        }

        located_params
    }

    fn push_inst(&mut self, block_id: OptId, inst: inst::Instruction, type_opt: Option<types::TypeInfo<'a>>) {
        let cur_fn = self.functions.get_mut(self.current_function_idx.unwrap()).unwrap();
        let blk_opt = cur_fn.blocks.get_mut(block_id.unwrap());
        match blk_opt {
            None => (),

            Some(blk) => {
                blk.insts.push(inst);

                match type_opt {
                    Some(type_info) => {
                        if let Some(current_type) = cur_fn.type_map.get(&type_info.var) {
                            if *current_type == type_info.typ {
                                cur_fn.type_map.insert(type_info.var, type_info.typ);
                            }
                        } else {
                            cur_fn.type_map.insert(type_info.var, type_info.typ);
                        }
                    },
                    None => ()
                }
            }
        }
    }

    fn set_term(&mut self, block_id: OptId, term: inst::Terminator) {
        let cur_fn = self.functions.get_mut(self.current_function_idx.unwrap()).unwrap();
        let blk_opt = cur_fn.blocks.get_mut(block_id.unwrap());
        match blk_opt {
            None => (),

            Some(blk) => {
                blk.term = Some(term);
            }
        }
    }

    fn get_new_var_id(&mut self) -> u32 {
        self.current_var_id += 1;
        self.current_var_id - 1
    }


    pub fn const_i64(&mut self, block_id: OptId, value: i64) -> u32 {
        let dst = self.get_new_var_id();
        let inst = inst::Instruction::ConstI64 { dst, value };
        self.push_inst(block_id, inst, Some(types::TypeInfo {
            var: dst,
            typ: types::Type::Int64
        }));
        dst
    }

    pub fn const_f64(&mut self, block_id: OptId, value: f64) -> u32 {
        let dst = self.get_new_var_id();
        let inst = inst::Instruction::ConstF64 { dst, value };
        self.push_inst(block_id, inst, Some(types::TypeInfo {
            var: dst,
            typ: types::Type::Float64
        }));
        dst
    }

    pub fn copy(&mut self, block_id: OptId, src: u32) -> u32 {
        let dst = self.get_new_var_id();
        let inst = inst::Instruction::Assign { dst, src };
        self.push_inst(block_id, inst, Some(types::TypeInfo {
            var: dst,
            typ: types::Type::None
        }));
        dst
    }

    pub fn add(&mut self, block_id: OptId, lhs: u32, rhs: u32) -> u32 {
        let dst = self.get_new_var_id();
        let inst = inst::Instruction::Add { dst, lhs, rhs };
        self.push_inst(block_id, inst, Some(types::TypeInfo {
            var: dst,
            typ: types::Type::None
        }));
        dst
    }

    pub fn ret(&mut self, block_id: OptId, src: u32) {
        let term = inst::Terminator::Return { src };
        self.set_term(block_id, term);
    }

    pub fn call(&mut self, block_id: OptId, callee: u32, args: Vec<u32>) -> u32 {
        let dst = self.get_new_var_id();
        let inst = inst::Instruction::Call { dst, callee, args };
        self.push_inst(block_id, inst, Some(types::TypeInfo {
            var: dst,
            typ: types::Type::None
        }));
        dst
    }

    pub fn load_builtin(&mut self, block_id: OptId, id: u32) -> u32 {
        let dst = self.get_new_var_id();
        let inst = inst::Instruction::LoadBuiltin { dst, id };
        self.push_inst(block_id, inst, Some(types::TypeInfo {
            var: dst,
            typ: types::Type::None
        }));
        dst
    }
}
