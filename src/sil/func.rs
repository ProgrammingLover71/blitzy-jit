use crate::sil::types;
use crate::sil::block;

#[derive(Debug)]
pub struct Function<'a> {
    pub name: String,
    pub params: Vec<types::Type<'a>>,
    pub returns: types::Type<'a>,
    pub blocks: Vec<block::Block<'a>>
}
