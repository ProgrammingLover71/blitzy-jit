use std::fmt;

use crate::parser::ast;
use crate::lexer::*;
use crate::error::*;


pub struct Parser<'a> {
    pub arena: &'a mut ast::NodeArena,
    
    lexer: lexer::Lexer,
    current: token::Token
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

    fn match_token(&mut self, t_type: token::TokenType, err: Error) -> Result<(), Error> {
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

        factor := INT | LPAREN expr RPAREN | IDENT

        call := factor (LPAREN (expr (COMMA expr)*)? RPAREN)?
        term := call (STAR|SLASH call)*
        expr := term (PLUS|MINUS term)*

        statement := RETURN expr NL
                  |= expr NL

        program := statement*
     */

    pub fn factor(&mut self) -> Result<ast::NodeId, Error> {
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

                self.match_token(token::TokenType::RParen, Error {
                    pe_line: self.current.t_line, 
                    pe_col:  self.current.t_col, 
                    pe_type: ErrorType::InvalidSyntaxError,
                    pe_msg: String::from("Expected closing parenthesis")
                })?;

                self.next();
                Ok(expr)
            }

            token::TokenType::Identifier => {
                let name = self.current.t_value.clone();
                self.next();

                Ok(self.arena.alloc(ast::AstNode {
                    n_line: self.current.t_line,
                    n_col:  self.current.t_col,
                    n_type: ast::AstNodeType::Ident { 
                        name 
                    }
                }))
            }

            _ => {
                Err(Error { 
                    pe_line: self.current.t_line, 
                    pe_col:  self.current.t_col, 
                    pe_type: ErrorType::InvalidSyntaxError,
                    pe_msg: String::from("Expected factor expression")
                })
            }
        }
    }


    pub fn call(&mut self) -> Result<ast::NodeId, Error> {
        let mut callee = self.factor()?;

        while self.current.t_type == token::TokenType::LParen {
            let line = self.current.t_line;
            let col = self.current.t_col;

            self.next();
            let mut args = Vec::new();

            if self.current.t_type != token::TokenType::RParen {
                loop {
                    let arg = self.expr()?;
                    args.push(arg);

                    if self.current.t_type == token::TokenType::RParen {
                        break;
                    }

                    self.match_token(token::TokenType::Comma, Error { 
                        pe_line: self.current.t_line, 
                        pe_col:  self.current.t_col, 
                        pe_type: ErrorType::InvalidSyntaxError,
                        pe_msg: String::from("Expected comma between function arguments")
                    })?;

                    self.next();
                }
            }

            self.match_token(token::TokenType::RParen, Error { 
                pe_line: line, 
                pe_col:  col, 
                pe_type: ErrorType::InvalidSyntaxError,
                pe_msg: String::from("Expected closing parenthesis after function call")
            })?;

            self.next();

            callee = self.arena.alloc(ast::AstNode {
                n_line: line,
                n_col:  col,
                n_type: ast::AstNodeType::Call { 
                    callee,
                    args
                }
            });
        }

        Ok(callee)
    }


    pub fn term(&mut self) -> Result<ast::NodeId, Error> {
        let mut left = self.call()?;

        while self.current.t_type == token::TokenType::Star || self.current.t_type == token::TokenType::Slash {
            let op = self.current.clone();
            self.next();

            let right = self.call()?;

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


    pub fn expr(&mut self) -> Result<ast::NodeId, Error> {
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


    pub fn statement(&mut self) -> Result<Option<ast::NodeId>, Error> {
        println!("Current token: {:?}", self.current);

        match self.current.t_type {
            token::TokenType::KReturn => {
                let line = self.current.t_line;
                let col = self.current.t_col;

                self.next();
                let expr = self.expr()?;

                self.match_token(TokenType::Newline, Error { 
                    pe_line: line, 
                    pe_col: col, 
                    pe_type: ErrorType::InvalidSyntaxError, 
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
                // Try to parse an expression statement
                let try_expr = self.expr();

                match try_expr {
                    Ok(expr) => {
                        self.match_token(TokenType::Newline, Error { 
                            pe_line: self.current.t_line, 
                            pe_col:  self.current.t_col, 
                            pe_type: ErrorType::InvalidSyntaxError,
                            pe_msg: String::from("Expected newline after expression statement")
                        })?;
                        Ok(Some(expr))
                    }

                    Err(_) => Err(Error { 
                        pe_line: self.current.t_line, 
                        pe_col:  self.current.t_col, 
                        pe_type: ErrorType::InvalidSyntaxError,
                        pe_msg: String::from("Expected statement")
                    })
                }
            }
        }
    }


    pub fn program(mut self) -> Result<ast::Program<'a>, Error> {
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