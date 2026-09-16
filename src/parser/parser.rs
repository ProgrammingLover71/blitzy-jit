use std::fmt;

use crate::parser::ast;
use crate::lexer::*;


pub struct Parser<'a> {
    pub arena: &'a mut ast::NodeArena,
    
    lexer: lexer::Lexer,
    current: token::Token
}


#[derive(Clone, Copy, Debug)]
pub enum ParseErrorType {
    InvalidSyntaxError
}

#[derive(Clone, Debug)]
pub struct ParseError {
    pub pe_line: u32,
    pub pe_col: u32,
    pub pe_type: ParseErrorType,
    pub pe_msg: String
}


impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} at line {}, col {}", self.pe_type, self.pe_msg, self.pe_line, self.pe_col)
    }
}

impl fmt::Display for ParseErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseErrorType::InvalidSyntaxError => write!(f, "InvalidSyntaxError"),
        }
    }
}


impl<'a> Parser<'a> {
    pub fn new(l: lexer::Lexer, arena: &'a mut ast::NodeArena) -> Self {
        let mut s = Self {
            arena,
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

    fn next(&mut self) {
        self.current = self.lexer.get_token();
    }

    fn current(&self) -> token::Token {
        self.current.clone()
    }

    fn match_token(&mut self, t_type: token::TokenType, err: ParseError) -> Result<(), ParseError> {
        if self.current.t_type == t_type {
            Ok(())
        } else {
            Err(err)
        }
    }

    fn check_token(&mut self, t_type: token::TokenType) -> bool {
        self.current.t_type == t_type
    }

    /*
        Expression rules:

        factor := INT | LPAREN expr RPAREN
        term := factor (STAR|SLASH factor)*
        expr := term (PLUS|MINUS term)*

        statement := RETURN expr NL

        program := statement*
     */

    pub fn factor(&mut self) -> Result<ast::NodeId, ParseError> {
        match self.current.t_type {
            token::TokenType::Int => {
                // .unwrap() here is safe, Int tokens always have a valid int associated with them
                let val = self.current.t_value.parse::<i64>().unwrap();
                self.next();

                Ok(self.arena.alloc(ast::AstNode {
                    n_line: self.current.t_line,
                    n_col: self.current.t_col,
                    n_type: ast::AstNodeType::IntLiteral { 
                        value: val 
                    }
                }))
            }

            token::TokenType::LParen => {
                self.next();
                let expr = self.expr()?;

                self.match_token(token::TokenType::RParen, ParseError {
                    pe_line: self.current.t_line, 
                    pe_col:  self.current.t_col, 
                    pe_type: ParseErrorType::InvalidSyntaxError,
                    pe_msg: String::from("Expected closing parenthesis")
                })?;

                self.next();
                Ok(expr)
            }

            _ => {
                Err(ParseError { 
                    pe_line: self.current.t_line, 
                    pe_col:  self.current.t_col, 
                    pe_type: ParseErrorType::InvalidSyntaxError,
                    pe_msg: String::from("Expected factor expression")
                })
            }
        }
    }


    pub fn term(&mut self) -> Result<ast::NodeId, ParseError> {
        let mut left = self.factor()?;

        while self.current.t_type == token::TokenType::Star || self.current.t_type == token::TokenType::Slash {
            let op = self.current.clone();
            self.next();

            let right = self.factor()?;

            left = self.arena.alloc(ast::AstNode {
                n_line: op.t_line,
                n_col:  op.t_col,
                n_type: ast::AstNodeType::BinaryOp {
                    left: left,
                    right: right,
                    op: ast::OpType::from_token(op.t_type)
                }
            });
        }
        
        Ok(left)
    }


    pub fn expr(&mut self) -> Result<ast::NodeId, ParseError> {
        let mut left = self.term()?;

        while self.current.t_type == token::TokenType::Plus || self.current.t_type == token::TokenType::Minus {
            let op = self.current.clone();
            self.next();

            let right = self.term()?;

            left = self.arena.alloc(ast::AstNode {
                n_line: op.t_line,
                n_col:  op.t_col,
                n_type: ast::AstNodeType::BinaryOp {
                    left: left,
                    right: right,
                    op: ast::OpType::from_token(op.t_type)
                }
            });
        }

        Ok(left)
    }


    pub fn statement(&mut self) -> Result<Option<ast::NodeId>, ParseError> {
        match self.current.t_type {
            token::TokenType::KReturn => {
                let line = self.current.t_line;
                let col = self.current.t_col;

                self.next();
                let expr = self.expr()?;

                self.match_token(TokenType::Newline, ParseError { 
                    pe_line: line, 
                    pe_col: col, 
                    pe_type: ParseErrorType::InvalidSyntaxError, 
                    pe_msg: String::from("Expected newline after return statement")
                })?;

                Ok(Some(self.arena.alloc(ast::AstNode {
                    n_line: line,
                    n_col:  col,
                    n_type: ast::AstNodeType::Return { 
                        value: expr 
                    }
                })))
            }

            TokenType::Newline => {
                self.next();
                self.statement()
            }

            TokenType::EndOfFile => {
                Ok(None)
            }

            _ => {
                Err(ParseError { 
                    pe_line: self.current.t_line, 
                    pe_col:  self.current.t_col, 
                    pe_type: ParseErrorType::InvalidSyntaxError,
                    pe_msg: String::from("Expected statement")
                })
            }
        }
    }


    pub fn program(&mut self) -> Result<ast::Program, ParseError> {
        let mut roots = Vec::new();

        while self.current.t_type != token::TokenType::EndOfFile {
            let stmt = self.statement()?;
            if let Some(stmt) = stmt {
                roots.push(stmt);
            }
        }

        Ok(ast::Program::new(self.arena, roots))
    }
}