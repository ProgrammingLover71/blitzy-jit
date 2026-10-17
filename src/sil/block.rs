use crate::sil::inst;
use crate::sil::types;

use std::fmt;

#[derive(Debug)]
pub struct Block<'a> {
    pub insts: Vec<inst::Instruction>,
    pub params: Vec<(u32, types::Type<'a>)>,
    pub term: Option<inst::Terminator>
}

impl<'a> fmt::Display for Block<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "blk {:?}", self.params)?;
        for inst in &self.insts {
            writeln!(f, "  {}", inst)?;
        }
        match self.term {
            Some(term) => write!(f, "  {}", term)?,
            None => write!(f, "")?,
        }

        Ok(())
    }
}
