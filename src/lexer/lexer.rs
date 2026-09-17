use crate::lexer::token::{Token, TokenType};

pub struct Lexer {
    pub source: String,

    current: char,
    index: isize,

    // Line/column data
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let mut new_source = source;
        new_source.push('\n');
        
        let mut s = Self {
            source: new_source,
            current: '\x00',
            index: -1,
            line: 1,
            col: 1,
        };

        s.next();
        s
    }

    fn next(&mut self) {
        self.index += 1;
        self.current = if self.index < self.source.len() as isize {
            self.source.as_bytes()[self.index as usize] as char
        } else {
            '\x00'
        };
    }

    pub fn get_token(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.col;

        if self.current == '\x00' {
            return Token {
                t_type: TokenType::EndOfFile,
                t_value: String::new(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
        }

        if self.current == ' ' || self.current == '\t' || self.current == '\r' {
            self.col += 1;
            self.next();
            return self.get_token();
        }

        if self.current == '\n' {
            self.line += 1;
            self.col = 1;
            let newline = Token {
                t_type: TokenType::Newline,
                t_value: "\n".to_string(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
            self.next();
            return newline;
        }

        if self.current == ';' {
            self.col += 1;
            let semicolon = Token {
                t_type: TokenType::Semicolon,
                t_value: ";".to_string(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
            self.next();
            return semicolon;
        }

        if self.current == '+' {
            self.col += 1;
            let plus = Token {
                t_type: TokenType::Plus,
                t_value: "+".to_string(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
            self.next();
            return plus;
        }

        if self.current == '-' {
            self.col += 1;
            let minus = Token {
                t_type: TokenType::Minus,
                t_value: "-".to_string(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
            self.next();
            return minus;
        }

        if self.current == '*' {
            self.col += 1;
            let star = Token {
                t_type: TokenType::Star,
                t_value: "*".to_string(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
            self.next();
            return star;
        }

        if self.current == '/' {
            self.col += 1;
            let slash = Token {
                t_type: TokenType::Slash,
                t_value: "/".to_string(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
            self.next();
            return slash;
        }

        if self.current == '(' {
            self.col += 1;
            let lparen = Token {
                t_type: TokenType::LParen,
                t_value: "(".to_string(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
            self.next();
            return lparen;
        }

        if self.current == ')' {
            self.col += 1;
            let rparen = Token {
                t_type: TokenType::RParen,
                t_value: ")".to_string(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
            self.next();
            return rparen;
        }

        if self.current == ',' {
            self.col += 1;
            let comma = Token {
                t_type: TokenType::Comma,
                t_value: ",".to_string(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
            self.next();
            return comma;
        }

        if self.current.is_ascii_digit() {
            let mut value = String::new();
            while self.current.is_ascii_digit() {
                value.push(self.current);
                self.col += 1;
                self.next();
            }

            return Token {
                t_type: TokenType::Int,
                t_value: value,
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
        }

        if self.current.is_ascii_alphabetic() || self.current == '_' {
            let mut value = String::new();
            while self.current.is_ascii_alphanumeric() || self.current == '_' {
                value.push(self.current);
                self.col += 1;
                self.next();
            }

            if value == "return" {
                return Token {
                    t_type: TokenType::KReturn,
                    t_value: value,
                    t_line: start_line as u32,
                    t_col: start_col as u32,
                };
            }

            return Token {
                t_type: TokenType::Identifier,
                t_value: value,
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
        }

        let ch = self.current.to_string();
        self.col += 1;
        self.next();
        Token {
            t_type: TokenType::Error,
            t_value: ch,
            t_line: start_line as u32,
            t_col: start_col as u32,
        }
    }
}