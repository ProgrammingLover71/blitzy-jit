use crate::error;
use crate::rt::bytecode;
use crate::rt::value;

#[derive(Clone)]
pub struct Frame<'a> {
    pub code: &'a bytecode::Block<'a>,
    pub ip: usize,
    pub regs: Vec<value::Value<'a>>,
}

impl<'a> Frame<'a> {
    pub fn new(code: &'a bytecode::Block) -> Self {
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

    fn pop(&mut self, frame: &Frame) -> Result<value::Value<'b>, error::Error> {
        self.stack
            .pop()
            .ok_or(error::Error::StackUnderflowError(frame.ip))
    }

    pub fn run(&mut self, code: &'b bytecode::Block) -> Result<value::Value<'b>, error::Error> {
        // Create a new frame and own it with `this_frame`
        let frame_index = self.frames.len();
        self.frames.push(Frame::new(code));
        let mut this_frame = self.frames[frame_index].clone();

        loop {
            match &this_frame.code.instructions[this_frame.ip] {
                bytecode::Opcode::Return { reg } => {
                    let reg_value = this_frame.regs[*reg as usize].clone();
                    return Ok(reg_value);
                }

                bytecode::Opcode::Call { dst, reg, nargs } => {
                    let reg_value = this_frame.regs[*reg as usize].clone();

                    let res = match reg_value {
                        value::Value::Function {
                            name: _,
                            code,
                            arity,
                        } => {
                            if arity != *nargs {
                                return Err(error::Error::ValueError(format!(
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

                    this_frame.regs[*dst as usize] = res?;
                }

                bytecode::Opcode::Load { dst, val } => {
                    this_frame.regs[*dst as usize] = val.clone();
                }

                bytecode::Opcode::Move { dst, src } => {
                    let src_value = this_frame.regs[*src as usize].clone();
                    this_frame.regs[*dst as usize] = src_value;
                }

                bytecode::Opcode::Push { reg } => {
                    let val = this_frame.regs[*reg as usize].clone();
                    self.push(val);
                }

                bytecode::Opcode::Pushi { imm } => {
                    self.push(imm.clone());
                }

                bytecode::Opcode::Pop { reg } => {
                    this_frame.regs[*reg as usize] = self.pop(&this_frame)?;
                }

                bytecode::Opcode::Add { dst, arg1, arg2 } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    let arg2_value = this_frame.regs[*arg2 as usize].clone();
                    this_frame.regs[*dst as usize] =
                        value::Value::add(arg1_value, arg2_value, &mut self.interner)?;
                }

                bytecode::Opcode::Addi { dst, arg1, imm } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    this_frame.regs[*dst as usize] =
                        value::Value::add(arg1_value, imm.clone(), &mut self.interner)?;
                }

                bytecode::Opcode::Sub { dst, arg1, arg2 } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    let arg2_value = this_frame.regs[*arg2 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::sub(arg1_value, arg2_value)?;
                }

                bytecode::Opcode::Subi { dst, arg1, imm } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::sub(arg1_value, imm.clone())?;
                }

                bytecode::Opcode::Mul { dst, arg1, arg2 } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    let arg2_value = this_frame.regs[*arg2 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::mul(arg1_value, arg2_value)?;
                }

                bytecode::Opcode::Muli { dst, arg1, imm } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::mul(arg1_value, imm.clone())?;
                }

                bytecode::Opcode::Div { dst, arg1, arg2 } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    let arg2_value = this_frame.regs[*arg2 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::div(arg1_value, arg2_value)?;
                }

                bytecode::Opcode::Divi { dst, arg1, imm } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::div(arg1_value, imm.clone())?;
                }

                bytecode::Opcode::Eq { dst, arg1, arg2 } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    let arg2_value = this_frame.regs[*arg2 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::eq(arg1_value, arg2_value)?;
                }

                bytecode::Opcode::Neq { dst, arg1, arg2 } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    let arg2_value = this_frame.regs[*arg2 as usize].clone();
                    this_frame.regs[*dst as usize] =
                        value::Value::not(value::Value::eq(arg1_value, arg2_value)?)?;
                }

                bytecode::Opcode::Gt { dst, arg1, arg2 } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    let arg2_value = this_frame.regs[*arg2 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::gt(arg1_value, arg2_value)?;
                }

                bytecode::Opcode::Gte { dst, arg1, arg2 } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    let arg2_value = this_frame.regs[*arg2 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::ge(arg1_value, arg2_value)?;
                }

                bytecode::Opcode::Lt { dst, arg1, arg2 } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    let arg2_value = this_frame.regs[*arg2 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::lt(arg1_value, arg2_value)?;
                }

                bytecode::Opcode::Lte { dst, arg1, arg2 } => {
                    let arg1_value = this_frame.regs[*arg1 as usize].clone();
                    let arg2_value = this_frame.regs[*arg2 as usize].clone();
                    this_frame.regs[*dst as usize] = value::Value::le(arg1_value, arg2_value)?;
                }

                bytecode::Opcode::Jmp { idx } => {
                    this_frame.ip = *idx;
                }
            }

            this_frame.ip += 1;
        }
    }
}
