use crate::error;
use crate::intern;
use crate::rt::bytecode;
use crate::rt::value;

#[derive(Clone)]
pub struct Frame<'a> {
    pub code: &'a bytecode::Block<'a>,
    pub ip: usize,
    pub regs: Vec<value::Value<'a>>,
}

impl<'a> Frame<'a> {
    pub fn new(code: &'a bytecode::Block<'a>) -> Self {
        Self {
            code,
            ip: 0,
            regs: vec![
                value::Value::None;
                bytecode::Block::find_used_regs(&code.instructions) as usize
            ],
        }
    }
}

#[derive(Clone)]
pub struct Interpreter<'b> {
    pub frames: Vec<Frame<'b>>,
    pub stack: Vec<value::Value<'b>>,
    pub interner: intern::Interner,
    pub globals: std::collections::HashMap<String, value::Value<'b>>,
}

impl<'b> Interpreter<'b> {
    pub fn new() -> Self {
        let mut interpreter = Self {
            frames: Vec::new(),
            stack: Vec::new(),
            interner: intern::Interner::new(),
            globals: std::collections::HashMap::new(),
        };

        interpreter.load_builtins();
        interpreter
    }

    fn push(&mut self, val: value::Value<'b>) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<value::Value<'b>, error::Error> {
        self.stack.pop().ok_or(error::Error {
            pe_line: 0,
            pe_col: 0,
            pe_type: error::ErrorType::StackUnderflowError,
            pe_msg: String::from("Stack underflow"),
        })
    }

    fn pop_at(&mut self, loc: bytecode::SourceLoc) -> Result<value::Value<'b>, error::Error> {
        self.pop().map_err(|err| Self::with_loc(loc, err))
    }

    fn with_loc(loc: bytecode::SourceLoc, mut err: error::Error) -> error::Error {
        if err.pe_line == 0 && err.pe_col == 0 {
            err.pe_line = loc.line;
            err.pe_col = loc.col;
        }
        err
    }

    fn error_at(loc: bytecode::SourceLoc, pe_type: error::ErrorType, pe_msg: String) -> error::Error {
        error::Error {
            pe_line: loc.line,
            pe_col: loc.col,
            pe_type,
            pe_msg,
        }
    }

    pub fn run(&mut self, code: &'b bytecode::Block<'b>) -> Result<value::Value<'b>, error::Error> {
        let frame_index = self.frames.len();
        self.frames.push(Frame::new(code));

        let result = loop {
            let frame_end = {
                let frame = &self.frames[frame_index];
                frame.ip >= frame.code.instructions.len()
            };
            if frame_end {
                break Ok(value::Value::None);
            }

            let current_ip = {
                let frame = &self.frames[frame_index];
                frame.ip
            };
            let current_loc = self.frames[frame_index]
                .code
                .location_for(current_ip)
                .unwrap_or(bytecode::SourceLoc::default());

            let instruction = {
                let frame = &self.frames[frame_index];
                frame.code.instructions[current_ip].clone()
            };

            {
                let frame = &mut self.frames[frame_index];
                frame.ip += 1;
            }

            match instruction {
                bytecode::Opcode::Return { reg } => {
                    let reg_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[reg as usize].clone()
                    };
                    break Ok(reg_value);
                }

                bytecode::Opcode::Call { dst, reg, nargs } => {
                    let reg_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[reg as usize].clone()
                    };

                    let result = match reg_value {
                        value::Value::Function { name: _, code, arity } => {
                            if arity != nargs {
                                return Err(Self::error_at(
                                    current_loc,
                                    error::ErrorType::ValueError,
                                    format!(
                                        "Invalid number of arguments: expected {}, got {}",
                                        arity, nargs
                                    ),
                                ));
                            }
                            self.run(code)
                        }

                        value::Value::NativeFunction { name: _, func, arity } => {
                            if arity != nargs {
                                return Err(Self::error_at(
                                    current_loc,
                                    error::ErrorType::ValueError,
                                    format!(
                                        "Invalid number of arguments: expected {}, got {}",
                                        arity, nargs
                                    ),
                                ));
                            }

                            let mut args = Vec::with_capacity(nargs as usize);
                            for _ in 0..nargs {
                                args.push(self.pop_at(current_loc)?);
                            }
                            args.reverse();

                            func(args)
                        }

                        value => Err(Self::error_at(
                            current_loc,
                            error::ErrorType::TypeError,
                            format!(
                                "Expected Callable value, got {}",
                                value::Value::type_of(value)
                            ),
                        )),
                    }?;

                    {
                        let frame = &mut self.frames[frame_index];
                        frame.regs[dst as usize] = result;
                    }
                }

                bytecode::Opcode::Load { dst, val } => {
                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = val.clone();
                }

                bytecode::Opcode::Move { dst, src } => {
                    let src_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[src as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = src_value;
                }

                bytecode::Opcode::Push { reg } => {
                    let value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[reg as usize].clone()
                    };
                    self.push(value);
                }

                bytecode::Opcode::Pushi { imm } => {
                    self.push(imm);
                }

                bytecode::Opcode::Pop { reg } => {
                    let value = self.pop_at(current_loc)?;
                    let frame = &mut self.frames[frame_index];
                    frame.regs[reg as usize] = value;
                }

                bytecode::Opcode::Add { dst, arg1, arg2 } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };
                    let arg2_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg2 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::add(arg1_value, arg2_value, &mut self.interner)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Addi { dst, arg1, imm } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::add(arg1_value, imm, &mut self.interner)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Sub { dst, arg1, arg2 } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };
                    let arg2_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg2 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::sub(arg1_value, arg2_value)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Subi { dst, arg1, imm } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::sub(arg1_value, imm)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Mul { dst, arg1, arg2 } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };
                    let arg2_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg2 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::mul(arg1_value, arg2_value)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Muli { dst, arg1, imm } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::mul(arg1_value, imm)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Div { dst, arg1, arg2 } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };
                    let arg2_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg2 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::div(arg1_value, arg2_value)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Divi { dst, arg1, imm } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::div(arg1_value, imm)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Eq { dst, arg1, arg2 } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };
                    let arg2_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg2 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::eq(arg1_value, arg2_value)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Neq { dst, arg1, arg2 } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };
                    let arg2_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg2 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::not(
                        value::Value::eq(arg1_value, arg2_value)
                            .map_err(|err| Self::with_loc(current_loc, err))?,
                    )
                    .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Gt { dst, arg1, arg2 } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };
                    let arg2_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg2 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::gt(arg1_value, arg2_value)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Gte { dst, arg1, arg2 } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };
                    let arg2_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg2 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::ge(arg1_value, arg2_value)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Lt { dst, arg1, arg2 } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };
                    let arg2_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg2 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::lt(arg1_value, arg2_value)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Lte { dst, arg1, arg2 } => {
                    let arg1_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg1 as usize].clone()
                    };
                    let arg2_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[arg2 as usize].clone()
                    };

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value::Value::le(arg1_value, arg2_value)
                        .map_err(|err| Self::with_loc(current_loc, err))?;
                }

                bytecode::Opcode::Jmp { idx } => {
                    let frame = &mut self.frames[frame_index];
                    frame.ip = idx;
                }

                bytecode::Opcode::LoadName { dst, name } => {
                    let value = self.globals.get(name).cloned().ok_or_else(|| {
                        Self::error_at(
                            current_loc,
                            error::ErrorType::NameError,
                            format!("Name '{}' is not defined", name),
                        )
                    })?;

                    let frame = &mut self.frames[frame_index];
                    frame.regs[dst as usize] = value;
                }

                bytecode::Opcode::StoreName { src, name } => {
                    let src_value = {
                        let frame = &self.frames[frame_index];
                        frame.regs[src as usize].clone()
                    };
                    self.globals.insert(name.to_string(), src_value);
                }
            }
        };

        self.frames.pop();
        result
    }

    fn load_builtins(&mut self) {
        let print_name = Box::leak(Box::new(String::from("print")));

        let builtins = vec![
            ("print", value::Value::NativeFunction {
                name: print_name,
                func: |args| {
                    for arg in args {
                        print!("{}", arg);
                    }
                    println!();
                    Ok(value::Value::None)
                },
                arity: 1,
            }),
        ];

        for (name, func) in builtins {
            self.globals.insert(name.to_string(), func);
        }
    }
}
