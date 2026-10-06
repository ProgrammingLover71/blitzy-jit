mod error;
mod lexer;
mod parser;

mod hir;
mod lir;
mod jit;

use lexer::*;
use parser::*;

use lir::LocationContext as LC;

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

        builder.const_i64(lir::Location::vreg(0, LC::Write), 7);
        builder.const_i64(lir::Location::vreg(1, LC::Write), 5);
        builder.add(
            lir::Location::vreg(2, LC::Write),
            lir::Location::vreg(0, LC::Read),
            lir::Location::vreg(1, LC::Read),
        );
        builder.return_(lir::Location::vreg(2, LC::Read));
    }

    println!("{:?}", builder.get_block(foo_b0));
}