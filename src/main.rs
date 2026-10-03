mod error;
mod lexer;
mod parser;

mod hir;
mod lir;
mod jit;

use lexer::*;
use parser::*;

fn main() {
    let mut builder = lir::LirBuilder::new();

    let foo_id = builder.start_function(
        String::from("foo"),
        Vec::new(),
        lir::LirType::I64
    );
    let foo_b0 = builder.start_block(foo_id, Vec::new());
    
    {
        builder.switch_block(foo_b0);

        builder.const_i64(lir::Location::reg(0, lir::LocationContext::Write), 7);
        builder.const_i64(lir::Location::reg(1, lir::LocationContext::Write), 5);
        builder.add(
            lir::Location::reg(2, lir::LocationContext::Write),
            lir::Location::reg(0, lir::LocationContext::Read),
            lir::Location::reg(1, lir::LocationContext::Read),
        );
        builder.return_(lir::Location::reg(2, lir::LocationContext::Read));
    }

    println!("{:?}", builder.get_block(foo_b0));
}