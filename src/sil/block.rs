use crate::sil::loc;
use crate::sil::inst;
use crate::sil::types;

use std::fmt::Error;
use std::fmt::Write;

#[derive(Debug)]
pub struct Block<'a> {
    pub insts: Vec<inst::Instruction>,
    pub params: Vec<(loc::Location, types::Type<'a>)>,
    pub term: Option<inst::Terminator>
}

impl<'a> Block<'a> {
    pub fn to_string(&self) -> Result<String, Error> {
        let mut s = String::new();

        writeln!(&mut s, "[params: {:?}]", self.params)?;
        for inst in &self.insts {
            writeln!(&mut s, "  {:?}", inst)?;
        }
        writeln!(&mut s, "  {:?}", self.term)?;

        Ok(s)
    }
}
