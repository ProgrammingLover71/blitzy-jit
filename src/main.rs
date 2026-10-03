mod error;
mod lexer;
mod parser;

mod hir;
mod jit;

use lexer::*;
use parser::*;

fn main() {
    let mut builder = hir::HirBuilder::new();

    let foo_id = builder.start_function(String::from("foo"), Vec::new());
    let foo_b0_id = builder.start_block(foo_id);
    builder.switch_block(foo_b0_id);

    {
        let five_id = builder.make_int(5);
        let seven_id = builder.make_int(7);
        let add_id = builder.make_binary_op(five_id, seven_id, hir::OpType::Add);
        builder.make_return(add_id);
    }
}