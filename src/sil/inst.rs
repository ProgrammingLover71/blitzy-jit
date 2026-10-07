use crate::sil::loc;


#[derive(Debug)]
pub enum Instruction {
    ConstI64 {
        dst: loc::Location,
        value: i64
    },

    ConstF64 {
        dst: loc::Location,
        value: f64
    },

    Assign {
        dst: loc::Location,
        src: loc::Location
    },

    Add {
        dst: loc::Location,
        lhs: loc::Location,
        rhs: loc::Location
    },

    Call {
        dst: loc::Location,
        callee: loc::Location,
        args: Vec<loc::Location>
    },

    LoadBuiltin {
        dst: loc::Location,
        id: u32
    },
}



