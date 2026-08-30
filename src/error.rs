use std::fmt;

pub enum Error {
    StackUnderflowError(usize),

    TypeError(String),
    ValueError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::StackUnderflowError(ip) => write!(f, "{}: At ip={}", Error::type_name(self), ip),
            Error::TypeError(msg) => write!(f, "{}: {}", Error::type_name(self), msg),
            Error::ValueError(msg) => write!(f, "{}: {}", Error::type_name(self), msg),
        }
    }
}

impl Error {
    fn type_name(err: &Error) -> String {
        match err {
            Error::StackUnderflowError(_) => String::from("StackUnderflowError"),
            Error::TypeError(_) => String::from("TypeError"),
            Error::ValueError(_) => String::from("ValueError"),
        }
    }
}
