mod intern;
mod error;
mod bytegen;

mod lexer;
mod parser;
mod rt;

use parser::ast;

fn main() {
    let src = String::from("print(5)\nprint(7 * 3 / 2.1)\n");

    let lex = lexer::Lexer::new(src);
    let mut arena = ast::NodeArena::new();
    let prs = parser::Parser::new(lex, &mut arena);
    let bgen = bytegen::Bytegen::new();

    let prog = prs.program();

    match prog {
        Ok(p) => {
            // println!("Parsed program: {:?}", p);
            let block = bgen.compile_program(&p);
            
            let mut interp = rt::interp::Interpreter::new();
            let result = interp.run(&block);

            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("Runtime error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
}
