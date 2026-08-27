use crate::rt::value;

pub type Register = u8;
pub type Value<'a> = value::Value<'a>;

pub enum Opcode<'a> {
    Return {
        reg: Register,
    },

    Call {
        dst: Register,
        reg: Register,
        nargs: u16,
    },

    Load {
        dst: Register,
        val: Value<'a>,
    },

    Move {
        dst: Register,
        src: Register,
    },

    Push {
        reg: Register,
    },

    Pushi {
        imm: Value<'a>,
    },

    Pop {
        reg: Register,
    },

    Add {
        dst: Register,
        arg1: Register,
        arg2: Register,
    },

    Addi {
        dst: Register,
        arg1: Register,
        imm: Value<'a>,
    },
}

pub struct Block<'a> {
    pub instructions: Vec<Opcode<'a>>,
    pub used_regs: u16,
}

impl<'a> Block<'a> {
    pub fn new(instructions: Vec<Opcode<'a>>) -> Self {
        Self {
            used_regs: Block::find_used_regs(&instructions),
            instructions,
        }
    }

    pub fn find_used_regs(insts: &[Opcode]) -> u16 {
        insts
            .iter()
            .map(|opcode| {
                let max_reg = match opcode {
                    Opcode::Return { reg } => *reg,
                    Opcode::Call { reg, .. } => *reg,
                    Opcode::Load { dst, .. } => *dst,
                    Opcode::Move { dst, src } => (*dst).max(*src),
                    Opcode::Push { reg } => *reg,
                    Opcode::Pushi { imm: _ } => 0,
                    Opcode::Pop { reg } => *reg,
                    Opcode::Add { dst, arg1, arg2 } => (*dst).max(*arg1).max(*arg2),
                    Opcode::Addi { dst, arg1, .. } => (*dst).max(*arg1),
                };

                u16::from(max_reg) + 1
            })
            .max()
            .unwrap_or(0)
    }
}
