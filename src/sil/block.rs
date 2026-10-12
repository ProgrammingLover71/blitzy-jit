use crate::sil::loc;
use crate::sil::inst;
use crate::sil::types;

#[derive(Debug)]
pub struct Block<'a> {
    pub insts: Vec<inst::Instruction>,
    pub params: Vec<(loc::Location, types::Type<'a>)>,
    pub term: Option<inst::Terminator>
}
