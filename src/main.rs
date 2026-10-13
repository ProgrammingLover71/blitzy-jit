mod error;
mod lexer;
mod parser;

mod sil;

use std::fmt::Error;

use sil::LocationContext as LC;

fn main() -> Result<(), Error> {
	let mut b = sil::ILBuilder::new();

	let foo_id = b.define_function(
		String::from("foo"), 
		vec![], 
		sil::Type::Int64
	);
	b.set_current_function(foo_id);

	let foo_blk0 = b.make_entry_block(foo_id);

	b.const_i64(foo_blk0, sil::Location::vreg(0, LC::Write), 5);
	b.const_i64(foo_blk0, sil::Location::vreg(1, LC::Write), 7);
	b.add(foo_blk0, 
		sil::Location::vreg(2, LC::Write), 
		sil::Location::vreg(0, LC::Read), 
		sil::Location::vreg(1, LC::Read));
	b.return_(foo_blk0, sil::Location::vreg(2, LC::Read));

	let foo = b.get_function(foo_id);

	println!("{}", foo.to_string()?);

	Ok(())
}
