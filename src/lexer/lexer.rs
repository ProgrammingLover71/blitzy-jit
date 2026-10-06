use crate::lexer::token::{Token, TokenType};
use std::collections::VecDeque;

pub struct Lexer {
    pub source: String,

    current: char,
    index: isize,

    // Line/column data
    line: usize,
    col: usize,
    indent_levels: Vec<usize>,
    pending_tokens: VecDeque<Token>,
    at_line_start: bool,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let mut new_source = source;
        if !new_source.ends_with('\n') {
            new_source.push('\n');
        }
        
        let mut s = Self {
            source: new_source,
            current: '\x00',
            index: -1,
            line: 1,
            col: 1,
            indent_levels: vec![0],
            pending_tokens: VecDeque::new(),
            at_line_start: true,
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
        if let Some(token) = self.pending_tokens.pop_front() {
            return token;
        }

        if self.at_line_start {
            let line = self.line;
            let mut indentation = 0;

            while self.current == ' ' || self.current == '\t' || self.current == '\r' {
                match self.current {
                    ' ' => {
                        indentation += 1;
                        self.col += 1;
                    }
                    '\t' => {
                        indentation += 4;
                        self.col += 1;
                    }
                    '\r' => self.col  = 1,
                    _ => unreachable!(),
                }
                self.next();
            }

            if self.current == '\n' {
                let newline = Token {
                    t_type: TokenType::Newline,
                    t_value: "\n".to_string(),
                    t_line: self.line as u32,
                    t_col: self.col as u32,
                };
                self.line += 1;
                self.col = 1;
                self.next();
                return newline;
            }

            if self.current == '\x00' {
                while self.indent_levels.len() > 1 {
                    self.indent_levels.pop();
                    self.pending_tokens.push_back(Token {
                        t_type: TokenType::Dedent,
                        t_value: String::new(),
                        t_line: line as u32,
                        t_col: 1,
                    });
                }
                self.at_line_start = false;
                self.pending_tokens.push_back(Token {
                    t_type: TokenType::EndOfFile,
                    t_value: String::new(),
                    t_line: line as u32,
                    t_col: self.col as u32,
                });
                return self.pending_tokens.pop_front().unwrap();
            }

            self.at_line_start = false;
            let current_indentation = *self.indent_levels.last().unwrap();
            if indentation > current_indentation {
                self.indent_levels.push(indentation);
                self.pending_tokens.push_back(Token {
                    t_type: TokenType::Indent,
                    t_value: String::new(),
                    t_line: line as u32,
                    t_col: 1,
                });
            } else if indentation < current_indentation {
                while indentation < *self.indent_levels.last().unwrap() {
                    self.indent_levels.pop();
                    self.pending_tokens.push_back(Token {
                        t_type: TokenType::Dedent,
                        t_value: String::new(),
                        t_line: line as u32,
                        t_col: 1,
                    });
                }

                if indentation != *self.indent_levels.last().unwrap() {
                    self.pending_tokens.push_back(Token {
                        t_type: TokenType::Error,
                        t_value: "Inconsistent indentation".to_string(),
                        t_line: line as u32,
                        t_col: 1,
                    });
                }
            }

            if let Some(token) = self.pending_tokens.pop_front() {
                return token;
            }
        }

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
            self.at_line_start = true;
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

        if self.current == ':' {
            self.col += 1;
            let colon = Token {
                t_type: TokenType::Colon,
                t_value: ":".to_string(),
                t_line: start_line as u32,
                t_col: start_col as u32,
            };
            self.next();
            return colon;
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

            match value {
                val if val == "return".to_string() => {
                    return Token {
                        t_type: TokenType::KReturn,
                        t_value: val,
                        t_line: start_line as u32,
                        t_col: start_col as u32,
                    };
                }

                val if val == "if".to_string() => {
                    return Token {
                        t_type: TokenType::KIf,
                        t_value: val,
                        t_line: start_line as u32,
                        t_col: start_col as u32,
                    };
                }

                val if val == "else".to_string() => {
                    return Token {
                        t_type: TokenType::KElse,
                        t_value: val,
                        t_line: start_line as u32,
                        t_col: start_col as u32,
                    };
                }
                
                _ => {
                    return Token {
                        t_type: TokenType::Identifier,
                        t_value: value,
                        t_line: start_line as u32,
                        t_col: start_col as u32,
                    };
                }
            }
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
