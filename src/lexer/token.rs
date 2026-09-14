use std::fmt;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    // Meta-tokens
    Error,
    EndOfFile,
    Indent, Dedent,

    // Statement separators
    Newline, Semicolon,
    
    // "Value" tokens
    Int,

    // Symbols
    Plus, Minus, Star, Slash,

    // Keywords
    KReturn,
}


pub struct Token {
    pub t_type: TokenType,
    pub t_value: String,
    pub t_line: u32,
    pub t_col: u32
}


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ type: {:?}", self.t_type);
        write!(f, ", value: {:?}", self.t_value);
        write!(f, ", line: {}", self.t_line);
        write!(f, ", column: {}", self.t_col);
        write!(f, " }}");
        Ok(())
    }
}