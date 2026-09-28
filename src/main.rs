mod intern;
mod error;
mod bytegen;
mod jit;

mod lexer;
mod parser;
mod rt;

use parser::ast;

fn jit_demo() {
    let block = rt::bytecode::Block::new(vec![
        rt::bytecode::Opcode::Load {
            dst: 0,
            val: rt::value::Value::Int(9),
            line: 1,
            col: 1,
        },
        rt::bytecode::Opcode::Load {
            dst: 1,
            val: rt::value::Value::Int(6),
            line: 1,
            col: 1,
        },
        rt::bytecode::Opcode::Add {
            dst: 2,
            arg1: 0,
            arg2: 1,
            line: 1,
            col: 1,
        },
        rt::bytecode::Opcode::Return {
            reg: 2,
            line: 2,
            col: 7,
        },
    ]);

    let method = rt::bytecode::JitMethod::compile(&block);
    match method.invoke() {
        Ok(val) => println!("JIT demo result: {}", val),
        Err(err) => println!("{}", err),
    }
}

fn main() {
    jit_demo();

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

#[cfg(test)]
mod tests {
    use crate::rt::{bytecode, interp, value};

    #[test]
    fn jit_method_round_trips_basic_math() {
        let block = bytecode::Block::new(vec![
            bytecode::Opcode::Load { dst: 0, val: value::Value::Int(4), line: 1, col: 1 },
            bytecode::Opcode::Load { dst: 1, val: value::Value::Int(3), line: 1, col: 1 },
            bytecode::Opcode::Add { dst: 2, arg1: 0, arg2: 1, line: 1, col: 1 },
            bytecode::Opcode::Move { dst: 3, src: 2, line: 1, col: 1 },
            bytecode::Opcode::Return { reg: 3, line: 2, col: 7 },
        ]);

        let method = bytecode::JitMethod::compile(&block);
        let result = method.invoke();

        assert!(matches!(result, Ok(value::Value::Int(7))));
    }
}
