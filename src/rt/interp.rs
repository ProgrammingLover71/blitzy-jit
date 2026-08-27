use crate::error;
use crate::rt::bytecode;
use crate::rt::value;

#[derive(Clone)]
pub struct Frame<'a> {
    pub code: &'a bytecode::Block<'a>,
    pub regs: Vec<value::Value<'a>>,
}

impl<'a> Frame<'a> {
    pub fn new(code: &'a bytecode::Block) -> Self {
        Self {
            code,
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
    pub interner: value::StringInterner,
}

impl<'b> Interpreter<'b> {
    pub fn new() -> Self {
        Self {
            frames: vec![],
            stack: vec![],
            interner: value::StringInterner::new(),
        }
    }

    fn push(&mut self, val: value::Value<'b>) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<value::Value<'b>, error::Error> {
        self.stack.pop().ok_or(error::Error::StackUnderflowError)
    }

    pub fn run(&mut self, code: &'b bytecode::Block) -> Result<value::Value<'b>, error::Error> {
        let mut ip = 0usize;

        // Set up an interpreter frame.
        let frame_index = self.frames.len();
        self.frames.push(Frame::new(code));

        loop {
            match &code.instructions[ip] {
                bytecode::Opcode::Return { reg } => {
                    let reg_value = self.frames[frame_index].regs[*reg as usize].clone();
                    return Ok(reg_value);
                }

                bytecode::Opcode::Call { dst, reg, nargs } => {
                    let reg_value = self.frames[frame_index].regs[*reg as usize].clone();

                    let res = match reg_value {
                        // Only allow functions to be called
                        // because who the fuck needs "hello"(2, 4)
                        value::Value::Function {
                            name: _,
                            code,
                            arity,
                        } => {
                            if arity != *nargs {
                                break Err(error::Error::ValueError(format!(
                                    "Invalid number of arguments: expected {}, got {}",
                                    arity, nargs
                                )));
                            }

                            self.run(code)
                        }

                        _ => Err(error::Error::TypeError(format!(
                            "Expected Callable value, got {}",
                            value::Value::type_of(reg_value)
                        ))),
                    };
                    self.frames[frame_index].regs[*dst as usize] = res?;
                }

                bytecode::Opcode::Load { dst, val } => {
                    self.frames[frame_index].regs[*dst as usize] = val.clone();
                }

                bytecode::Opcode::Move { dst, src } => {
                    let src_value = self.frames[frame_index].regs[*src as usize].clone();
                    self.frames[frame_index].regs[*dst as usize] = src_value;
                }

                bytecode::Opcode::Push { reg } => {
                    let val = self.frames[frame_index].regs[*reg as usize].clone();
                    self.push(val);
                }

                bytecode::Opcode::Pushi { imm } => {
                    self.push(imm.clone());
                }

                bytecode::Opcode::Pop { reg } => {
                    self.frames[frame_index].regs[*reg as usize] = self.pop()?;
                }

                bytecode::Opcode::Add { dst, arg1, arg2 } => {
                    let arg1_value = self.frames[frame_index].regs[*arg1 as usize].clone();
                    let arg2_value = self.frames[frame_index].regs[*arg2 as usize].clone();
                    self.frames[frame_index].regs[*dst as usize] =
                        value::Value::add(arg1_value, arg2_value, &mut self.interner)?;
                }

                bytecode::Opcode::Addi { dst, arg1, imm } => {
                    let arg1_value = self.frames[frame_index].regs[*arg1 as usize].clone();
                    self.frames[frame_index].regs[*dst as usize] =
                        value::Value::add(arg1_value, imm.clone(), &mut self.interner)?;
                }
            }

            ip += 1;
        }
    }
}
