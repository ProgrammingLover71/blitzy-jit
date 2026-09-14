use std::fmt;


pub enum TokenType {
    // Meta-tokens
    Error,
    EndOfFile,
    Indent, Dedent,

    // Statement separators
    Newline, Semicolon,
    
    // "Value" tokens
    Int,

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
    fn fmt(&self, f: &mut fmt::Formatter) -> Result {
        write!(f, "{ type: ");
    }
}