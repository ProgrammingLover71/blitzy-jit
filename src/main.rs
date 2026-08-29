mod error;
mod rt;
use rt::*;

fn main() {
    let foo_name = String::from("foo");
    let foo_code = bytecode::Block::new(vec![
        bytecode::Opcode::Pop { reg: 0 },
        bytecode::Opcode::Pop { reg: 1 },
        bytecode::Opcode::Eq {
            dst: 0,
            arg1: 0,
            arg2: 1,
        },
        bytecode::Opcode::Return { reg: 0 },
    ]);

    let block = bytecode::Block::new(vec![
        bytecode::Opcode::Load {
            dst: 0,
            val: value::Value::Float(1.2),
        },
        bytecode::Opcode::Load {
            dst: 1,
            val: value::Value::Float(1.2),
        },
        bytecode::Opcode::Push { reg: 0 },
        bytecode::Opcode::Push { reg: 1 },
        bytecode::Opcode::Load {
            dst: 2,
            val: value::Value::Function {
                name: &foo_name,
                code: &foo_code,
                arity: 2,
            },
        },
        bytecode::Opcode::Call {
            dst: 0,
            reg: 2,
            nargs: 2,
        },
        bytecode::Opcode::Return { reg: 0 },
    ]);

    let mut interp = interp::Interpreter::new();
    let result = interp.run(&block);

    match result {
        Ok(val) => println!("{}", val),
        Err(err) => println!("{}", err),
    }

    println!("Number of used registers: {}", block.used_regs);
}
