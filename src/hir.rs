#[derive(Debug)]
pub enum OpType {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HirNodeId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HirBlockId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HirFunctionId(pub u32);

#[derive(Debug)]
pub enum HirNode {
    IntLiteral {
        value: i64
    },

    Ident {
        name: String
    },

    BinaryOp {
        left: HirNodeId,
        right: HirNodeId,
        op: OpType
    },

    Call {
        callee: HirNodeId,
        args: Vec<HirNodeId>
    },

    Return {
        value: HirNodeId
    },

    Branch {
        condition: HirNodeId,
        then_branch: HirBlockId,
        else_branch: Option<HirBlockId>
    },
}

#[derive(Debug)]
pub struct HirBlock {
    pub nodes: Vec<HirNodeId>,
    pub roots: Vec<HirNodeId>
}

#[derive(Debug)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<(String, HirType)>,
    pub returns: HirType,
    pub block_ids: Vec<HirBlockId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HirType {
    Int,
    Float,
    Any
}

#[derive(Debug)]
pub struct HirProgram {
    pub funcs: Vec<HirFunction>,
    pub blocks: Vec<HirBlock>,
    pub nodes: Vec<HirNode>
}


pub struct HirBuilder {
    program: HirProgram,
    current_block: Option<HirBlockId>,
}

impl HirBuilder {
    pub fn new() -> Self {
        Self {
            program: HirProgram {
                funcs: Vec::new(),
                blocks: Vec::new(),
                nodes: Vec::new(),
            },
            current_block: None,
        }
    }

    pub fn start_function(&mut self, name: String, params: Vec<(String, HirType)>, returns: HirType) -> HirFunctionId {
        let func = HirFunction {
            name,
            params,
            returns,
            block_ids: Vec::new()
        };
        self.program.funcs.push(func);
        HirFunctionId((self.program.funcs.len() - 1) as u32)
    }

    pub fn start_block(&mut self, fn_id: HirFunctionId) -> HirBlockId {
        let block = HirBlock {
            nodes: Vec::new(),
            roots: Vec::new()
        };
        self.program.blocks.push(block);

        let block_id = HirBlockId((self.program.blocks.len() - 1) as u32);

        self.program.funcs[fn_id.0 as usize].block_ids.push(block_id);
        block_id
    }

    pub fn switch_block(&mut self, block_id: HirBlockId) {
        self.current_block = Some(block_id);
    }

    pub fn switch_function(&mut self, fn_id: HirFunctionId) {
        if let Some(block_id) = self.program.funcs[fn_id.0 as usize].block_ids.first() {
            self.current_block = Some(*block_id);
        } else {
            self.current_block = None;
        }
    }

    pub fn get_block(&self, block_id: HirBlockId) -> Option<&HirBlock> {
        self.program.blocks.get(block_id.0 as usize)
    }

    pub fn get_function(&self, fn_id: HirFunctionId) -> Option<&HirFunction> {
        self.program.funcs.get(fn_id.0 as usize)
    }

    pub fn get_program(&self) -> &HirProgram {
        &self.program
    }

    fn push_node(&mut self, node_id: HirNodeId) {
        if let Some(block_id) = self.current_block {
            self.program.blocks[block_id.0 as usize].nodes.push(node_id);
        }
    }

    fn push_root(&mut self, node_id: HirNodeId) {
        if let Some(block_id) = self.current_block {
            self.program.blocks[block_id.0 as usize].roots.push(node_id);
        }
    }



    pub fn make_int(&mut self, value: i64) -> HirNodeId {
        let node = HirNode::IntLiteral { value };

        self.program.nodes.push(node);
        let id = HirNodeId((self.program.nodes.len() - 1) as u32);
        self.push_node(id);

        id
    }

    pub fn make_ident(&mut self, name: String) -> HirNodeId {
        let node = HirNode::Ident { name };

        
        self.program.nodes.push(node);
        let id = HirNodeId((self.program.nodes.len() - 1) as u32);
        self.push_node(id);

        id
    }

    pub fn make_binary_op(&mut self, left: HirNodeId, right: HirNodeId, op: OpType) -> HirNodeId {
        let node = HirNode::BinaryOp { left, right, op };

        self.program.nodes.push(node);
        let id = HirNodeId((self.program.nodes.len() - 1) as u32);
        self.push_node(id);

        id
    }

    pub fn make_call(&mut self, callee: HirNodeId, args: Vec<HirNodeId>) -> HirNodeId {
        let node = HirNode::Call { callee, args };

        self.program.nodes.push(node);
        let id = HirNodeId((self.program.nodes.len() - 1) as u32);
        self.push_node(id);

        id
    }

    pub fn make_return(&mut self, value: HirNodeId) -> HirNodeId {
        let node = HirNode::Return { value };

        self.program.nodes.push(node);
        let id = HirNodeId((self.program.nodes.len() - 1) as u32);
        self.push_node(id);
        self.push_root(id);

        id
    }

    pub fn make_branch(&mut self, condition: HirNodeId, then_branch: HirBlockId, else_branch: Option<HirBlockId>) -> HirNodeId {
        let node = HirNode::Branch { condition, then_branch, else_branch };

        self.program.nodes.push(node);
        let id = HirNodeId((self.program.nodes.len() - 1) as u32);
        self.push_node(id);

        id
    }
}
