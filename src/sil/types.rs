#[derive(Debug, Clone, Copy)]
pub enum Type<'a> {
    Int64,
    Float64,
    Box(&'a Type<'a>)
}