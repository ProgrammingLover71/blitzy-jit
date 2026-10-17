use std::fmt;


#[derive(Debug)]
pub enum Instruction {
    ConstI64 {
        dst: u32,
        value: i64
    },

    ConstF64 {
        dst: u32,
        value: f64
    },

    Assign {
        dst: u32,
        src: u32
    },

    Add {
        dst: u32,
        lhs: u32,
        rhs: u32
    },

    Call {
        dst: u32,
        callee: u32,
        args: Vec<u32>
    },

    LoadBuiltin {
        dst: u32,
        id: u32
    },
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Terminator {
    Return {
        src: u32
    }
}


impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConstI64 { dst, value } => write!(f, "%{} = const.i64 {}", dst, value)?,
            Self::ConstF64 { dst, value } => write!(f, "%{} = const.f64 {}", dst, value)?,
            Self::Add { dst, lhs, rhs } => write!(f, "%{} = add %{}, %{}", dst, lhs, rhs)?,
            Self::Assign { dst, src } => write!(f, "%{} = %{}", dst, src)?,
            Self::Call { dst, callee, args } => write!(f, "%{} = call %{} {:?}", dst, callee, args)?,
            Self::LoadBuiltin { dst, id } => write!(f, "%{} = ldbltin {}", dst, id)?,
        }

        Ok(())
    }
}


impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return { src } => write!(f, "ret %{}", src)?,
        }

        Ok(())
    }
}
