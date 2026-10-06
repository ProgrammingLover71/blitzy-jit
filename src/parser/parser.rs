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
        println!("Token: {:?}", self.current);
    }

    fn match_token(&mut self, t_type: token::TokenType, err: Error) -> Result<(), Error> {
        if self.current.t_type == t_type {
            Ok(())
        } else {
            Err(err)
        }
    }

    fn check_token(&self, t_type: token::TokenType) -> bool {
        self.current.t_type == t_type
    }

    /*
        Expression rules:

        factor := INT | LPAREN expr RPAREN | IDENT

        call := factor (LPAREN (expr (COMMA expr)*)? RPAREN)?
        term := call (STAR|SLASH call)*
        expr := term (PLUS|MINUS term)*

        statement := RETURN expr NL
                  |= IF expr COLON NL INDENT statement* DEDENT (ELSE COLON NL INDENT statement* DEDENT)?
                  |= expr NL

        program := statement*
     */

    pub fn factor(&mut self) -> Result<ast::NodeId, Error> {
        match self.current.t_type {
            token::TokenType::Int => {
                // .unwrap() here is safe, Int tokens always have a valid int associated with them
                let val = self.current.t_value.parse::<i64>();
                let i_val = match val {
                    Ok(int_val) => int_val,
                    Err(_) => {
                        return Err(Error {
                            pe_line: self.current.t_line,
                            pe_col:  self.current.t_col,
                            pe_type: ErrorType::InvalidLiteralError,
                            pe_msg:  format!(
                                "int literal out of range: {}",
                                self.current.t_value
                            )
                        });
                    }
                };
                self.next();

                let ln = self.current.t_line;
                let col = self.current.t_col;

                Ok(self.arena.alloc(ast::AstNode {
                    n_line: ln,
                    n_col:  col,
                    n_type: ast::AstNodeType::IntLiteral { 
                        value: i_val 
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
                    pe_msg:  String::from("Expected closing parenthesis")
                })?;

                self.next();
                Ok(expr)
            }

            token::TokenType::Identifier => {
                let name = self.current.t_value.clone();
                self.next();

                let ln = self.current.t_line;
                let col = self.current.t_col;

                Ok(self.arena.alloc(ast::AstNode {
                    n_line: ln,
                    n_col:  col,
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
                    pe_msg:  String::from("Expected factor expression")
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
                self.parse_return_statement()
            }

            token::TokenType::KIf => {
                self.parse_if_statement()
            }

            token::TokenType::Newline => {
                while self.check_token(TokenType::Newline) {
                    self.next();
                }
                self.statement()
            }

            token::TokenType::EndOfFile => {
                Ok(None)
            }

            _ => {
                // Try to parse an expression statement
                let try_expr = self.expr();

                match try_expr {
                    Ok(expr) => {
                        self.match_token(token::TokenType::Newline, Error { 
                            pe_line: self.current.t_line, 
                            pe_col:  self.current.t_col, 
                            pe_type: ErrorType::InvalidSyntaxError,
                            pe_msg: String::from("Expected newline after expression statement")
                        })?;
                        self.next();
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


    fn parse_return_statement(&mut self) -> Result<Option<ast::NodeId>, Error> {
        let line = self.current.t_line;
        let col = self.current.t_col;
    
        self.next();
        let expr = self.expr()?;
    
        self.match_token(TokenType::Newline, Error { 
            pe_line: line, 
            pe_col:  col, 
            pe_type: ErrorType::InvalidSyntaxError, 
            pe_msg:  String::from("Expected newline after return statement")
        })?;
        self.next();
    
        Ok(Some(self.arena.alloc(ast::AstNode {
            n_line: line,
            n_col:  col,
            n_type: ast::AstNodeType::Return { 
                value: expr 
            }
        })))
    }


    fn parse_if_statement(&mut self) -> Result<Option<ast::NodeId>, Error> {
        let line = self.current.t_line;
        let col = self.current.t_col;

        self.next();
        let condition = self.expr()?;

        self.match_token(TokenType::Colon, Error { 
            pe_line: self.current.t_line, 
            pe_col:  self.current.t_col, 
            pe_type: ErrorType::InvalidSyntaxError, 
            pe_msg: String::from("Expected colon after if condition")
        })?;

        self.next();
        self.match_token(TokenType::Newline, Error { 
            pe_line: self.current.t_line, 
            pe_col:  self.current.t_col, 
            pe_type: ErrorType::InvalidSyntaxError, 
            pe_msg:  String::from("Expected newline after if statement")
        })?;
        self.next();

        let then_branch = self.parse_block()?;
        let mut else_branch = None;

        if self.check_token(TokenType::KElse) {
            self.next();
            self.match_token(TokenType::Colon, Error { 
                pe_line: self.current.t_line, 
                pe_col:  self.current.t_col, 
                pe_type: ErrorType::InvalidSyntaxError, 
                pe_msg:  String::from("Expected colon after else")
            })?;

            self.next();
            self.match_token(TokenType::Newline, Error { 
                pe_line: self.current.t_line, 
                pe_col:  self.current.t_col, 
                pe_type: ErrorType::InvalidSyntaxError, 
                pe_msg:  String::from("Expected newline after else")
            })?;
            self.next();

            else_branch = Some(self.parse_block()?);
        }
        
        Ok(Some(self.arena.alloc(ast::AstNode {
            n_line: line,
            n_col:  col,
            n_type: ast::AstNodeType::If { 
                condition,
                then_branch,
                else_branch
            }
        })))
    }


    fn parse_block(&mut self) -> Result<ast::NodeId, Error> {
        self.match_token(TokenType::Indent, Error { 
            pe_line: self.current.t_line, 
            pe_col:  self.current.t_col, 
            pe_type: ErrorType::InvalidSyntaxError, 
            pe_msg:  String::from("Expected indentation for block")
        })?;

        self.next();
        let mut statements = Vec::new();

        while !self.check_token(TokenType::Dedent) && !self.check_token(TokenType::EndOfFile) {
            if let Some(stmt) = self.statement()? {
                statements.push(stmt);
            }
        }

        self.match_token(TokenType::Dedent, Error { 
            pe_line: self.current.t_line, 
            pe_col:  self.current.t_col, 
            pe_type: ErrorType::InvalidSyntaxError, 
            pe_msg:  String::from("Expected dedentation after block")
        })?;

        self.next();

        Ok(self.arena.alloc(ast::AstNode {
            n_line: self.current.t_line,
            n_col:  self.current.t_col,
            n_type: ast::AstNodeType::Block { 
                statements
            }
        }))
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
