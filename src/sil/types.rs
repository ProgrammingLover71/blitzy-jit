#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Type<'a> {
    Int64,
    Float64,
    Box(&'a Type<'a>),
    None
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TypeInfo<'a> {
    pub var: u32,
    pub typ: Type<'a>
}