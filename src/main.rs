mod error;
mod lexer;
mod parser;

mod lir;
mod sil;

use lexer::*;
use parser::*;

use lir::LocationContext as LC;

fn main() {
    let a: sil::types::Type = sil::types::Type::Int64;
}