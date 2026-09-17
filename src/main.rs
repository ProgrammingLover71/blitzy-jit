mod intern;
mod error;
mod bytegen;

mod lexer;
mod parser;
mod rt;

use parser::ast;

fn main() {
    let src = String::from("print(5)\n");

    let lex = lexer::Lexer::new(src);
    let mut arena = ast::NodeArena::new();
    let prs = parser::Parser::new(lex, &mut arena);
    let bgen = bytegen::Bytegen::new();

    let prog = prs.program();

    match prog {
        Ok(p) => {
            println!("Parsed program: {:?}", p);
            let block = bgen.compile_program(&p);
            
            let mut interp = rt::interp::Interpreter::new();
            let result = interp.run(&block);

            match result {
                Ok(val) => {
                    println!("Program result: {}", val);
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
