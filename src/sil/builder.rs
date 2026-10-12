use crate::sil::loc;
use crate::sil::func;
use crate::sil::inst;
use crate::sil::types;

pub struct ILBuilder<'a> {
    functions: Vec<func::Function<'a>>,
    pub current_function_idx: i32
}

impl<'a> ILBuilder<'a> {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            current_function_idx: -1    // -1 = Not applicable
        }
    }

    pub fn define_function(&mut self, name: String, params: Vec<types::Type<'a>>, ret: types::Type<'a>) -> i32 {
        let func = func::Function {
            name, params, returns: ret,
            blocks: Vec::new()
        };
        self.functions.push(func);
        return (self.functions.len() - 1) as i32;
    }


    pub fn const_i64(&mut self, dst: loc::Location, value: i64) {
        let node = inst::Instruction::ConstI64 { dst, value };
        // self.push_node(node);
    }

    pub fn const_f64(&mut self, dst: loc::Location, value: f64) {
        let node = inst::Instruction::ConstF64 { dst, value };
        // self.push_node(node);
    }

    pub fn assign(&mut self, dst: loc::Location, src: loc::Location) {
        let node = inst::Instruction::Assign { dst, src };
        // self.push_node(node);
    }

    pub fn add(&mut self, dst: loc::Location, lhs: loc::Location, rhs: loc::Location) {
        let node = inst::Instruction::Add { dst, lhs, rhs };
        // self.push_node(node);
    }

    pub fn return_(&mut self, src: loc::Location) {
        let term = inst::Terminator::Return { src };
        // self.set_term(term);
    }

    pub fn call(&mut self, dst: loc::Location, callee: loc::Location, args: Vec<loc::Location>) {
        let node = inst::Instruction::Call { dst, callee, args };
        // self.push_node(node);
    }

    pub fn load_builtin(&mut self, dst: loc::Location, id: u32) {
        let node = inst::Instruction::LoadBuiltin { dst, id };
        // self.push_node(node);
    }
}
