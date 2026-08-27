use std::fmt;

pub enum Error {
    StackUnderflowError,

    TypeError(String),
    ValueError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::StackUnderflowError => write!(f, "{}", Error::type_name(self)),
            Error::TypeError(msg) => write!(f, "{}: {}", Error::type_name(self), msg),
            Error::ValueError(msg) => write!(f, "{}: {}", Error::type_name(self), msg),
        }
    }
}

impl Error {
    fn type_name(err: &Error) -> String {
        match err {
            Error::StackUnderflowError => String::from("[VM] StackUnderflowError"),
            Error::TypeError(_) => String::from("TypeError"),
            Error::ValueError(_) => String::from("ValueError"),
        }
    }
}
