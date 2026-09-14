use crate::lexer::token;

pub struct Lexer {
    pub source: String,
    
    current: char,
    index: isize,
    
    // Line/column data
    line: usize,
    col: usize
}


impl Lexer {
    fn new(source: String) -> Self {
        let mut s = Self {
            source,
            current: '\x00',
            index: -1,
            line: 1,
            col: 0
        };
    
        s.next();
        s
    }

    fn next(&mut self) {
        self.index += 1;
        self.current = if self.index <= self.source.len() { 
            self.source.as_bytes()[self.index] as char
        } else {
            '\x00'
        };
    }

    fn get_token
}