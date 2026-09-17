use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::parser::ast;
use crate::rt::bytecode;
use crate::rt::value;

pub struct Bytegen<'a> {
    next_reg: u8,
    instructions: Vec<bytecode::Opcode<'a>>,
}

impl<'a> Bytegen<'a> {
    pub fn new() -> Self {
        Self {
            next_reg: 0,
            instructions: Vec::new(),
        }
    }

    pub fn compile_program(mut self, program: &ast::Program<'a>) -> bytecode::Block<'a> {
        for root in &program.roots {
            self.compile_node(program, *root);
        }

        let instructions = self.allocate_registers();
        bytecode::Block::new(instructions)
    }

    fn compile_node(&mut self, program: &ast::Program<'a>, node_id: ast::NodeId) -> u8 {
        let node = program.get_node(node_id).unwrap();

        match &node.n_type {
            ast::AstNodeType::IntLiteral { value } => {
                let dst = self.next_reg;
                self.next_reg += 1;

                self.instructions.push(bytecode::Opcode::Load {
                    dst,
                    val: value::Value::Int(*value),
                });

                dst
            }

            ast::AstNodeType::BinaryOp { left, right, op } => {
                let left_reg = self.compile_node(program, *left);
                let right_reg = self.compile_node(program, *right);
                let dst = self.next_reg;
                self.next_reg += 1;

                match op {
                    ast::OpType::Add => {
                        self.instructions.push(bytecode::Opcode::Add {
                            dst,
                            arg1: left_reg,
                            arg2: right_reg,
                        });
                    }

                    ast::OpType::Subtract => {
                        self.instructions.push(bytecode::Opcode::Sub {
                            dst,
                            arg1: left_reg,
                            arg2: right_reg,
                        });
                    }

                    ast::OpType::Multiply => {
                        self.instructions.push(bytecode::Opcode::Mul {
                            dst,
                            arg1: left_reg,
                            arg2: right_reg,
                        });
                    }

                    ast::OpType::Divide => {
                        self.instructions.push(bytecode::Opcode::Div {
                            dst,
                            arg1: left_reg,
                            arg2: right_reg,
                        });
                    }
                }

                dst
            }

            ast::AstNodeType::Ident { name } => {
                let dst = self.next_reg;
                self.next_reg += 1;

                self.instructions.push(bytecode::Opcode::Load {
                    dst,
                    val: value::Value::String(Arc::new(name.as_str())),
                });

                dst
            }

            ast::AstNodeType::Call { callee, args } => {
                let callee_reg = self.compile_node(program, *callee);
                let mut arg_regs = Vec::new();

                for arg in args {
                    let arg_reg = self.compile_node(program, *arg);
                    arg_regs.push(arg_reg);
                }

                let dst = self.next_reg;
                self.next_reg += 1;

                self.instructions.push(bytecode::Opcode::Call {
                    dst,
                    reg: callee_reg,
                    nargs: arg_regs.len() as u16,
                });

                dst
            }

            ast::AstNodeType::Return { value } => {
                let reg = self.compile_node(program, *value);
                self.instructions.push(bytecode::Opcode::Return { reg });
                reg
            }
        }
    }

    fn allocate_registers(&mut self) -> Vec<bytecode::Opcode<'a>> {
        let max_reg = self.next_reg as usize;
        let mut adjacency: Vec<Vec<u8>> = vec![Vec::new(); max_reg];
        let mut move_bias: HashMap<u8, u8> = HashMap::new();
        let mut live: HashSet<u8> = HashSet::new();

        for idx in (0..self.instructions.len()).rev() {
            let inst = &self.instructions[idx];
            let defs = def_registers(inst);
            let uses = use_registers(inst);

            for def in &defs {
                for reg in &live {
                    if reg != def {
                        let edge = *reg;
                        if !adjacency[*def as usize].contains(&edge) {
                            adjacency[*def as usize].push(edge);
                        }
                        if !adjacency[edge as usize].contains(def) {
                            adjacency[edge as usize].push(*def);
                        }
                    }
                }
            }

            for reg in &uses {
                if !defs.contains(reg) {
                    live.insert(*reg);
                }
            }

            for reg in &defs {
                live.remove(reg);
            }

            if let bytecode::Opcode::Move { dst, src } = inst {
                if dst != src {
                    move_bias.insert(*dst, *src);
                }
            }
        }

        let mut colors: Vec<u8> = vec![0; max_reg];
        let mut order: Vec<u8> = (0..self.next_reg).collect();
        order.sort_by(|a, b| {
            let left = adjacency[*a as usize].len();
            let right = adjacency[*b as usize].len();
            right.cmp(&left)
        });

        for reg in order {
            let mut used = vec![false; max_reg];
            for &neighbor in &adjacency[reg as usize] {
                used[colors[neighbor as usize] as usize] = true;
            }

            let preferred = move_bias.get(&reg).copied();
            let mut color = match preferred {
                Some(pref) if !used[pref as usize] => pref,
                _ => 0,
            };

            if preferred.is_some() && !used[color as usize] {
                // prefer the move source color when it is legal
            } else {
                while used[color as usize] {
                    color = color.saturating_add(1);
                }
            }

            colors[reg as usize] = color;
        }

        let mut remapped: Vec<bytecode::Opcode<'a>> = Vec::with_capacity(self.instructions.len());

        for inst in self.instructions.drain(..) {
            remapped.push(remap_instruction(inst, &colors));
        }

        remapped
    }
}

fn remap_instruction<'a>(inst: bytecode::Opcode<'a>, colors: &[u8]) -> bytecode::Opcode<'a> {
    match inst {
        bytecode::Opcode::Return { reg } => bytecode::Opcode::Return {
            reg: remap_reg(reg, colors),
        },

        bytecode::Opcode::Call { dst, reg, nargs } => bytecode::Opcode::Call {
            dst: remap_reg(dst, colors),
            reg: remap_reg(reg, colors),
            nargs,
        },

        bytecode::Opcode::Load { dst, val } => bytecode::Opcode::Load {
            dst: remap_reg(dst, colors),
            val,
        },

        bytecode::Opcode::Move { dst, src } => {
            let dst = remap_reg(dst, colors);
            let src = remap_reg(src, colors);
            if dst == src {
                bytecode::Opcode::Move { dst, src }
            } else {
                bytecode::Opcode::Move { dst, src }
            }
        }

        bytecode::Opcode::Push { reg } => bytecode::Opcode::Push {
            reg: remap_reg(reg, colors),
        },

        bytecode::Opcode::Pushi { imm } => bytecode::Opcode::Pushi { imm },

        bytecode::Opcode::Pop { reg } => bytecode::Opcode::Pop {
            reg: remap_reg(reg, colors),
        },

        bytecode::Opcode::Add { dst, arg1, arg2 } => bytecode::Opcode::Add {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            arg2: remap_reg(arg2, colors),
        },

        bytecode::Opcode::Addi { dst, arg1, imm } => bytecode::Opcode::Addi {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            imm,
        },

        bytecode::Opcode::Sub { dst, arg1, arg2 } => bytecode::Opcode::Sub {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            arg2: remap_reg(arg2, colors),
        },

        bytecode::Opcode::Subi { dst, arg1, imm } => bytecode::Opcode::Subi {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            imm,
        },

        bytecode::Opcode::Mul { dst, arg1, arg2 } => bytecode::Opcode::Mul {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            arg2: remap_reg(arg2, colors),
        },

        bytecode::Opcode::Muli { dst, arg1, imm } => bytecode::Opcode::Muli {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            imm,
        },

        bytecode::Opcode::Div { dst, arg1, arg2 } => bytecode::Opcode::Div {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            arg2: remap_reg(arg2, colors),
        },

        bytecode::Opcode::Divi { dst, arg1, imm } => bytecode::Opcode::Divi {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            imm,
        },

        bytecode::Opcode::Eq { dst, arg1, arg2 } => bytecode::Opcode::Eq {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            arg2: remap_reg(arg2, colors),
        },

        bytecode::Opcode::Neq { dst, arg1, arg2 } => bytecode::Opcode::Neq {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            arg2: remap_reg(arg2, colors),
        },

        bytecode::Opcode::Gt { dst, arg1, arg2 } => bytecode::Opcode::Gt {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            arg2: remap_reg(arg2, colors),
        },

        bytecode::Opcode::Lt { dst, arg1, arg2 } => bytecode::Opcode::Lt {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            arg2: remap_reg(arg2, colors),
        },

        bytecode::Opcode::Gte { dst, arg1, arg2 } => bytecode::Opcode::Gte {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            arg2: remap_reg(arg2, colors),
        },

        bytecode::Opcode::Lte { dst, arg1, arg2 } => bytecode::Opcode::Lte {
            dst: remap_reg(dst, colors),
            arg1: remap_reg(arg1, colors),
            arg2: remap_reg(arg2, colors),
        },

        bytecode::Opcode::Jmp { idx } => bytecode::Opcode::Jmp { idx },
    }
}

fn remap_reg(reg: u8, colors: &[u8]) -> u8 {
    colors[reg as usize]
}

fn def_registers(inst: &bytecode::Opcode) -> Vec<u8> {
    match inst {
        bytecode::Opcode::Return { .. } => Vec::new(),
        bytecode::Opcode::Call { dst, .. } => vec![*dst],
        bytecode::Opcode::Load { dst, .. } => vec![*dst],
        bytecode::Opcode::Move { dst, .. } => vec![*dst],
        bytecode::Opcode::Push { .. } => Vec::new(),
        bytecode::Opcode::Pushi { .. } => Vec::new(),
        bytecode::Opcode::Pop { reg } => vec![*reg],
        bytecode::Opcode::Add { dst, .. }
        | bytecode::Opcode::Addi { dst, .. }
        | bytecode::Opcode::Sub { dst, .. }
        | bytecode::Opcode::Subi { dst, .. }
        | bytecode::Opcode::Mul { dst, .. }
        | bytecode::Opcode::Muli { dst, .. }
        | bytecode::Opcode::Div { dst, .. }
        | bytecode::Opcode::Divi { dst, .. }
        | bytecode::Opcode::Eq { dst, .. }
        | bytecode::Opcode::Neq { dst, .. }
        | bytecode::Opcode::Gt { dst, .. }
        | bytecode::Opcode::Lt { dst, .. }
        | bytecode::Opcode::Gte { dst, .. }
        | bytecode::Opcode::Lte { dst, .. } => vec![*dst],
        bytecode::Opcode::Jmp { .. } => Vec::new(),
    }
}

fn use_registers(inst: &bytecode::Opcode) -> Vec<u8> {
    match inst {
        bytecode::Opcode::Return { reg } => vec![*reg],
        bytecode::Opcode::Call { reg, .. } => vec![*reg],
        bytecode::Opcode::Load { .. } => Vec::new(),
        bytecode::Opcode::Move { dst: _, src } => vec![*src],
        bytecode::Opcode::Push { reg } => vec![*reg],
        bytecode::Opcode::Pushi { .. } => Vec::new(),
        bytecode::Opcode::Pop { .. } => Vec::new(),
        bytecode::Opcode::Add { arg1, arg2, .. }
        | bytecode::Opcode::Sub { arg1, arg2, .. }
        | bytecode::Opcode::Mul { arg1, arg2, .. }
        | bytecode::Opcode::Div { arg1, arg2, .. }
        | bytecode::Opcode::Eq { arg1, arg2, .. }
        | bytecode::Opcode::Neq { arg1, arg2, .. }
        | bytecode::Opcode::Gt { arg1, arg2, .. }
        | bytecode::Opcode::Lt { arg1, arg2, .. }
        | bytecode::Opcode::Gte { arg1, arg2, .. }
        | bytecode::Opcode::Lte { arg1, arg2, .. } => vec![*arg1, *arg2],
        bytecode::Opcode::Addi { arg1, .. }
        | bytecode::Opcode::Subi { arg1, .. }
        | bytecode::Opcode::Muli { arg1, .. }
        | bytecode::Opcode::Divi { arg1, .. } => vec![*arg1],
        bytecode::Opcode::Jmp { .. } => Vec::new(),
    }
}
