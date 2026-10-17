mod error;
mod lexer;
mod parser;

mod sil;

use std::fmt::Error;

fn main() -> Result<(), Error> {
	let mut b = sil::ILBuilder::new();

	let foo_id = b.define_function(
		String::from("foo"), 
		vec![], 
		sil::Type::Int64
	);
	b.set_current_function(foo_id);

	let foo_bb0 = b.make_entry_block(foo_id);

	{
		let v0 = b.const_i64(foo_bb0, 5);
		let v1 = b.const_i64(foo_bb0, 7);
		let v2 = b.add(foo_bb0, v0, v1);
		let v3 = b.load_builtin(foo_bb0, 0u32);
		let v4 = b.call(foo_bb0, v3, vec![v2]);
		b.ret(foo_bb0, v4);
	}

	let foo = b.get_function(foo_id);

	println!("{}", foo);

	Ok(())
}
