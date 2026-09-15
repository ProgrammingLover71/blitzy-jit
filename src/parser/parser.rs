use crate::parser::ast;
use crate::lexer::*;


pub struct Parser {
    pub arena: ast::NodeArena,
    
    lexer: lexer::Lexer,
    current: token::Token
}


#[derive(Clone, Copy)]
pub enum ParseErrorType {
    InvalidSyntaxError
}

#[derive(Clone)]
pub struct ParseError {
    pub pe_line: u32,
    pub pe_col: u32,
    pub pe_type: ParseErrorType,
    pub pe_msg: String
}


impl Parser {
    pub fn new(l: lexer::Lexer) -> Self {
        let mut s = Self {
            arena: ast::NodeArena::new(),
            lexer: l,
            current: Token {
                t_type: TokenType::EndOfFile,
                t_value: String::new(),
                t_line: 0,
                t_col: 0,
            }
        };
        s.next();
        s
    }

    pub fn next(&mut self) {
        self.current = self.lexer.get_token();
    }

    pub fn current(&self) -> token::Token {
        self.current.clone()
    }

    /*
        Expression rules:

        factor := INT
        term := factor (STAR|SLASH factor)*
        expr := term (PLUS|MINUS term)*
     */

    pub fn factor(&mut self) -> Result<ast::AstNode, ParseError> {
        match self.current.t_type {
            token::TokenType::Int => {
                // .unwrap() here is safe, Int tokens always have a valid int associated with them
                let val = self.current.t_value.parse::<i64>().unwrap();
                self.next();

                Ok(ast::AstNode {
                    n_line: self.current.t_line,
                    n_col: self.current.t_col,
                    n_type: ast::AstNodeType::IntLiteral { 
                        value: val 
                    }
                })
            }

            _ => {
                Err(ParseError { 
                    pe_line: self.current.t_line, 
                    pe_col: self.current.t_col, 
                    pe_type: ParseErrorType::InvalidSyntaxError,
                    pe_msg: String::from("Expected factor expression")
                })
            }
        }
    }


    pub fn term(&mut self) -> Result<ast::AstNode, ParseError> {
        let mut left = self.factor()?;

        // TODO: figure out how to check for either * or /, and
        // also find a way to use the node arena
        
        Ok(left)
    }
}