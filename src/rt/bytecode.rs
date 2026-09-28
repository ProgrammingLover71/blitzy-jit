use crate::rt::value;

pub type Register = u8;
pub type Value<'a> = value::Value<'a>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: u32,
    pub col: u32,
}

#[derive(Debug, Clone)]
pub enum Opcode<'a> {
    Return {
        reg: Register,
        line: u32,
        col: u32,
    },

    Call {
        dst: Register,
        reg: Register,
        nargs: u16,
        line: u32,
        col: u32,
    },

    Load {
        dst: Register,
        val: Value<'a>,
        line: u32,
        col: u32,
    },

    Move {
        dst: Register,
        src: Register,
        line: u32,
        col: u32,
    },

    Ldloc {
        dst: Register,
        local: u16,
        line: u32,
        col: u32,
    },

    Stloc {
        local: u16,
        reg: Register,
        line: u32,
        col: u32,
    },

    Push {
        reg: Register,
        line: u32,
        col: u32,
    },

    Pushi {
        imm: Value<'a>,
        line: u32,
        col: u32,
    },

    Pop {
        reg: Register,
        line: u32,
        col: u32,
    },

    Add {
        dst: Register,
        arg1: Register,
        arg2: Register,
        line: u32,
        col: u32,
    },

    Addi {
        dst: Register,
        arg1: Register,
        imm: Value<'a>,
        line: u32,
        col: u32,
    },

    Sub {
        dst: Register,
        arg1: Register,
        arg2: Register,
        line: u32,
        col: u32,
    },

    Subi {
        dst: Register,
        arg1: Register,
        imm: Value<'a>,
        line: u32,
        col: u32,
    },

    Mul {
        dst: Register,
        arg1: Register,
        arg2: Register,
        line: u32,
        col: u32,
    },

    Muli {
        dst: Register,
        arg1: Register,
        imm: Value<'a>,
        line: u32,
        col: u32,
    },

    Div {
        dst: Register,
        arg1: Register,
        arg2: Register,
        line: u32,
        col: u32,
    },

    Divi {
        dst: Register,
        arg1: Register,
        imm: Value<'a>,
        line: u32,
        col: u32,
    },

    Eq {
        dst: Register,
        arg1: Register,
        arg2: Register,
        line: u32,
        col: u32,
    },

    Neq {
        dst: Register,
        arg1: Register,
        arg2: Register,
        line: u32,
        col: u32,
    },

    Gt {
        dst: Register,
        arg1: Register,
        arg2: Register,
        line: u32,
        col: u32,
    },

    Lt {
        dst: Register,
        arg1: Register,
        arg2: Register,
        line: u32,
        col: u32,
    },

    Gte {
        dst: Register,
        arg1: Register,
        arg2: Register,
        line: u32,
        col: u32,
    },

    Lte {
        dst: Register,
        arg1: Register,
        arg2: Register,
        line: u32,
        col: u32,
    },

    Jmp {
        idx: usize,
        line: u32,
        col: u32,
    },
}

impl<'a> Opcode<'a> {
    #[inline]
    pub fn location(&self) -> SourceLocation {
        match self {
            Opcode::Return { line, col, .. }
            | Opcode::Call { line, col, .. }
            | Opcode::Load { line, col, .. }
            | Opcode::Move { line, col, .. }
            | Opcode::Ldloc { line, col, .. }
            | Opcode::Stloc { line, col, .. }
            | Opcode::Push { line, col, .. }
            | Opcode::Pushi { line, col, .. }
            | Opcode::Pop { line, col, .. }
            | Opcode::Add { line, col, .. }
            | Opcode::Addi { line, col, .. }
            | Opcode::Sub { line, col, .. }
            | Opcode::Subi { line, col, .. }
            | Opcode::Mul { line, col, .. }
            | Opcode::Muli { line, col, .. }
            | Opcode::Div { line, col, .. }
            | Opcode::Divi { line, col, .. }
            | Opcode::Eq { line, col, .. }
            | Opcode::Neq { line, col, .. }
            | Opcode::Gt { line, col, .. }
            | Opcode::Lt { line, col, .. }
            | Opcode::Gte { line, col, .. }
            | Opcode::Lte { line, col, .. }
            | Opcode::Jmp { line, col, .. } => SourceLocation {
                line: *line,
                col: *col,
            },
        }
    }

    #[inline]
    fn max_reg(&self) -> u8 {
        match self {
            Opcode::Return { reg, .. } => *reg,
            Opcode::Call { reg, .. } => *reg,
            Opcode::Load { dst, .. } => *dst,
            Opcode::Move { dst, src, .. } => (*dst).max(*src),
            Opcode::Ldloc { dst, .. } => *dst,
            Opcode::Stloc { reg, .. } => *reg,
            Opcode::Push { reg, .. } => *reg,
            Opcode::Pushi { .. } => 0,
            Opcode::Pop { reg, .. } => *reg,
            Opcode::Add { dst, arg1, arg2, .. } => (*dst).max(*arg1).max(*arg2),
            Opcode::Addi { dst, arg1, .. } => (*dst).max(*arg1),
            Opcode::Sub { dst, arg1, arg2, .. } => (*dst).max(*arg1).max(*arg2),
            Opcode::Subi { dst, arg1, .. } => (*dst).max(*arg1),
            Opcode::Mul { dst, arg1, arg2, .. } => (*dst).max(*arg1).max(*arg2),
            Opcode::Muli { dst, arg1, .. } => (*dst).max(*arg1),
            Opcode::Div { dst, arg1, arg2, .. } => (*dst).max(*arg1).max(*arg2),
            Opcode::Divi { dst, arg1, .. } => (*dst).max(*arg1),
            Opcode::Eq { dst, arg1, arg2, .. } => (*dst).max(*arg1).max(*arg2),
            Opcode::Neq { dst, arg1, arg2, .. } => (*dst).max(*arg1).max(*arg2),
            Opcode::Gt { dst, arg1, arg2, .. } => (*dst).max(*arg1).max(*arg2),
            Opcode::Lt { dst, arg1, arg2, .. } => (*dst).max(*arg1).max(*arg2),
            Opcode::Gte { dst, arg1, arg2, .. } => (*dst).max(*arg1).max(*arg2),
            Opcode::Lte { dst, arg1, arg2, .. } => (*dst).max(*arg1).max(*arg2),
            Opcode::Jmp { .. } => 0,
        }
    }

    #[inline]
    fn max_local(&self) -> u16 {
        match self {
            Opcode::Ldloc { local, .. } | Opcode::Stloc { local, .. } => *local + 1,
            _ => 0,
        }
    }
}

#[derive(Debug)]
pub struct Block<'a> {
    pub instructions: Vec<Opcode<'a>>,
    pub used_regs: u16,
    pub used_locals: u16,
}

impl<'a> Block<'a> {
    pub fn new(instructions: Vec<Opcode<'a>>) -> Self {
        Self {
            used_regs: Block::find_used_regs(&instructions),
            used_locals: Block::find_used_locals(&instructions),
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

    pub fn find_used_locals(insts: &[Opcode<'a>]) -> u16 {
        insts
            .iter()
            .map(Opcode::max_local)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Clone)]
pub struct JitMethod<'a> {
    pub code: &'a Block<'a>,
    jit: crate::jit::tier1::Tier1Jit,
    entry: unsafe extern "C" fn() -> i64,
}

impl<'a> JitMethod<'a> {
    pub fn compile(code: &'a Block<'a>) -> Self {
        let mut jit = crate::jit::tier1::Tier1Jit::new();
        let entry = jit.compile_block(code).expect("Tier1 JIT compilation failed");
        Self { code, jit, entry }
    }

    pub fn invoke(&self) -> Result<crate::rt::value::Value<'a>, crate::error::Error> {
        let result = unsafe { (self.entry)() };
        Ok(crate::rt::value::Value::Int(result))
    }
}
