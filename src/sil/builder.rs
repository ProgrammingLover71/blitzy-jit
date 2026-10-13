use crate::sil::block;
use crate::sil::loc;
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
    current_block_idx: OptId
}

impl<'a> ILBuilder<'a> {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            current_function_idx: OptId::None,
            current_block_idx: OptId::None
        }
    }

    pub fn define_function(&mut self, name: String, params: Vec<types::Type<'a>>, ret: types::Type<'a>) -> OptId {
        let func = func::Function {
            name, params, returns: ret,
            blocks: Vec::new()
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
        let func = self.functions.get_mut(function_id.unwrap()).unwrap();
                
        func.blocks.push(block::Block {
            insts: Vec::new(),
            params: Self::assign_params_to_locations(&func.params),
            term: None
        });

        OptId::Id(self.functions.len() - 1)
    }

    fn assign_params_to_locations(params: &Vec<types::Type<'a>>) -> Vec<(loc::Location, types::Type<'a>)> {
        let mut located_params = Vec::new();
        let vreg_index = 0;

        for type_param in params {
            located_params.push((
                loc::Location::vreg(
                    vreg_index, 
                    loc::LocationContext::Write
                ), 
                type_param.clone()
            ));
        }

        located_params
    }

    fn push_inst(&mut self, block_id: OptId, inst: inst::Instruction) {
        let cur_fn = self.functions.get_mut(self.current_function_idx.unwrap()).unwrap();
        let blk_opt = cur_fn.blocks.get_mut(block_id.unwrap());
        match blk_opt {
            None => (),

            Some(blk) => {
                blk.insts.push(inst);
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


    pub fn const_i64(&mut self, block_id: OptId, dst: loc::Location, value: i64) {
        let inst = inst::Instruction::ConstI64 { dst, value };
        self.push_inst(block_id, inst);
    }

    pub fn const_f64(&mut self, block_id: OptId, dst: loc::Location, value: f64) {
        let inst = inst::Instruction::ConstF64 { dst, value };
        self.push_inst(block_id, inst);
    }

    pub fn assign(&mut self, block_id: OptId, dst: loc::Location, src: loc::Location) {
        let inst = inst::Instruction::Assign { dst, src };
        self.push_inst(block_id, inst);
    }

    pub fn add(&mut self, block_id: OptId, dst: loc::Location, lhs: loc::Location, rhs: loc::Location) {
        let inst = inst::Instruction::Add { dst, lhs, rhs };
        self.push_inst(block_id, inst);
    }

    pub fn return_(&mut self, block_id: OptId, src: loc::Location) {
        let term = inst::Terminator::Return { src };
        self.set_term(block_id, term);
    }

    pub fn call(&mut self, block_id: OptId, dst: loc::Location, callee: loc::Location, args: Vec<loc::Location>) {
        let inst = inst::Instruction::Call { dst, callee, args };
        self.push_inst(block_id, inst);
    }

    pub fn load_builtin(&mut self, block_id: OptId, dst: loc::Location, id: u32) {
        let inst = inst::Instruction::LoadBuiltin { dst, id };
        self.push_inst(block_id, inst);
    }
}
