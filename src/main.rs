mod error;
mod lexer;
mod parser;

mod sil;

fn main() {
    let mut b = sil::ILBuilder::new();

    {
        let foo_id = b.define_function(
            String::from("foo"), 
            vec![sil::Type::Int64], 
            sil::Type::Int64
        );

        let foo_blk0 = b.make_entry_block(foo_id);
    }
}
