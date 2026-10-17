use crate::sil::types;
use crate::sil::block;

use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Function<'a> {
    pub name: String,
    pub params: Vec<types::Type<'a>>,
    pub returns: types::Type<'a>,
    pub blocks: Vec<block::Block<'a>>,

    pub type_map: HashMap<u32, types::Type<'a>>
}


impl<'a> fmt::Display for Function<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fn {}{:?} -> {:?}", self.name, self.params, self.returns)?;
        writeln!(f, "types: {:?}", self.type_map)?;

        for blk in &self.blocks {
            writeln!(f, "{}", blk)?;
        }
        writeln!(f, "endf")?;

        Ok(())
    }
}
