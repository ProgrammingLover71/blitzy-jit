use crate::sil::types;
use crate::sil::block;

use std::fmt::Error;
use std::fmt::Write;

#[derive(Debug)]
pub struct Function<'a> {
    pub name: String,
    pub params: Vec<types::Type<'a>>,
    pub returns: types::Type<'a>,
    pub blocks: Vec<block::Block<'a>>
}


impl<'a> Function<'a> {
    pub fn to_string(&self) -> Result<String, Error> {
        let mut s = String::new();

        writeln!(&mut s, "function {}\n- params: {:?}\n- return: {:?}\n{{", self.name, self.params, self.returns)?;
        for blk in &self.blocks {
            writeln!(&mut s, "{}", blk.to_string()?)?;
        }
        writeln!(&mut s, "}}")?;

        Ok(s)
    }
}
