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

    Sub {
        dst: Register,
        arg1: Register,
        arg2: Register,
    },

    Subi {
        dst: Register,
        arg1: Register,
        imm: Value<'a>,
    },

    Mul {
        dst: Register,
        arg1: Register,
        arg2: Register,
    },

    Muli {
        dst: Register,
        arg1: Register,
        imm: Value<'a>,
    },

    Div {
        dst: Register,
        arg1: Register,
        arg2: Register,
    },

    Divi {
        dst: Register,
        arg1: Register,
        imm: Value<'a>,
    },

    Eq {
        dst: Register,
        arg1: Register,
        arg2: Register
    },

    Neq {
        dst: Register,
        arg1: Register,
        arg2: Register
    },

    Gt {
        dst: Register,
        arg1: Register,
        arg2: Register
    },

    Lt {
        dst: Register,
        arg1: Register,
        arg2: Register
    },

    Gte {
        dst: Register,
        arg1: Register,
        arg2: Register
    },

    Lte {
        dst: Register,
        arg1: Register,
        arg2: Register
    },

    Jmp {
        idx: usize
    }
}

pub struct Block<'a> {
    pub instructions: Vec<Opcode<'a>>,
    pub used_regs: u16,
}

impl<'a> Opcode<'a> {
    #[inline]
    fn max_reg(&self) -> u8 {
        match self {
            Opcode::Return { reg } 
                => *reg,
            Opcode::Call { reg, .. } 
                => *reg,

            // Data transfer
            Opcode::Load { dst, .. } 
                => *dst,
            Opcode::Move { dst, src } 
                => (*dst).max(*src),

            // Push/pop
            Opcode::Push { reg } 
                => *reg,
            Opcode::Pushi { .. } 
                => 0,
            Opcode::Pop { reg } 
                => *reg,

            // Math ops
            Opcode::Add { dst, arg1, arg2 } 
                => (*dst).max(*arg1).max(*arg2),
            Opcode::Addi { dst, arg1, .. } 
                => (*dst).max(*arg1),
            Opcode::Sub { dst, arg1, arg2 } 
                => (*dst).max(*arg1).max(*arg2),
            Opcode::Subi { dst, arg1, .. } 
                => (*dst).max(*arg1),
            Opcode::Mul { dst, arg1, arg2 } 
                => (*dst).max(*arg1).max(*arg2),
            Opcode::Muli { dst, arg1, .. } 
                => (*dst).max(*arg1),
            Opcode::Div { dst, arg1, arg2 } 
                => (*dst).max(*arg1).max(*arg2),
            Opcode::Divi { dst, arg1, .. } 
                => (*dst).max(*arg1),

            // Boolean ops
            Opcode::Eq { dst, arg1, arg2 } 
                => (*dst).max(*arg1).max(*arg2),
            Opcode::Neq { dst, arg1, arg2 } 
                => (*dst).max(*arg1).max(*arg2),
            Opcode::Gt { dst, arg1, arg2 } 
                => (*dst).max(*arg1).max(*arg2),
            Opcode::Lt { dst, arg1, arg2 } 
                => (*dst).max(*arg1).max(*arg2),
            Opcode::Gte { dst, arg1, arg2 } 
                => (*dst).max(*arg1).max(*arg2),
            Opcode::Lte { dst, arg1, arg2 } 
                => (*dst).max(*arg1).max(*arg2),

            // Jumps
            Opcode::Jmp { .. } 
                => 0,
        }
    }
}

impl<'a> Block<'a> {
    pub fn new(instructions: Vec<Opcode<'a>>) -> Self {
        Self {
            used_regs: Block::find_used_regs(&instructions),
            instructions,
        }
    }

    pub fn find_used_regs(insts: &[Opcode<'a>]) -> u16 {
        insts
            .iter()
            .map(Opcode::max_reg)
            .max()
            .map_or(0, |max_reg| u16::from(max_reg) + 1)
    }
}
