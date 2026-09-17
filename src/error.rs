use std::fmt;


#[derive(Clone, Copy, Debug)]
pub enum ErrorType {
    InvalidSyntaxError,

    StackUnderflowError,
    TypeError,
    ValueError,
}

#[derive(Clone, Debug)]
pub struct Error {
    pub pe_line: u32,
    pub pe_col: u32,
    pub pe_type: ErrorType,
    pub pe_msg: String
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} at line {}, col {}", self.pe_type, self.pe_msg, self.pe_line, self.pe_col)
    }
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorType::InvalidSyntaxError => write!(f, "InvalidSyntaxError"),
            ErrorType::StackUnderflowError => write!(f, "StackUnderflowError"),
            ErrorType::TypeError => write!(f, "TypeError"),
            ErrorType::ValueError => write!(f, "ValueError"),
            _ => unreachable!()
        }
    }
}
