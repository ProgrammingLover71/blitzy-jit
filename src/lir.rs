#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocationType {
    VReg(u8),
    Local(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocationContext {
    Read,
    Write
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub l_type: LocationType,
    pub ctx: LocationContext,
}

impl Location {
    pub fn new(l_type: LocationType, ctx: LocationContext) -> Location {
        Self { l_type, ctx }
    }

    pub fn vreg(n: u8, ctx: LocationContext) -> Location {
        Self { l_type: LocationType::VReg(n), ctx }
    }

    pub fn local(n: u32, ctx: LocationContext) -> Location {
        Self { l_type: LocationType::Local(n), ctx }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LirNodeId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LirBlockId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LirFunctionId(pub u32);

#[derive(Debug)]
pub enum LirNode {
    ConstI64 {
        dst: Location,
        value: i64
    },

    ConstF64 {
        dst: Location,
        value: f64
    },

    Assign {
        dst: Location,
        src: Location
    },

    Add {
        dst: Location,
        lhs: Location,
        rhs: Location
    },

    Call {
        dst: Location,
        callee: Location,
        args: Vec<Location>
    },

    LoadBuiltin {
        dst: Location,
        id: u32
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LirTerminator {
    Return {
        src: Location
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LirType {
    I64,
    F64
}

#[derive(Debug)]
pub struct LirBlock {
    pub nodes: Vec<LirNodeId>,
    pub params: Vec<(Location, LirType)>,
    pub term: Option<LirTerminator>
}

#[derive(Debug)]
pub struct LirFunction {
    pub name: String,
    pub params: Vec<LirType>,
    pub returns: LirType,
    pub block_ids: Vec<LirBlockId>
}

#[derive(Debug)]
pub struct LirProgram {
    pub functions: Vec<LirFunction>,
    pub blocks: Vec<LirBlock>,
    pub nodes: Vec<LirNode>
}


pub struct LirBuilder {
    pub program: LirProgram,
    pub current_block: Option<LirBlockId>
}

impl LirBuilder {
    pub fn new() -> Self {
        Self {
            program: LirProgram {
                functions: Vec::new(),
                blocks: Vec::new(),
                nodes: Vec::new()
            },
            current_block: None
        }
    }

    pub fn start_function(&mut self, name: String, params: Vec<LirType>, returns: LirType) -> LirFunctionId {
        let func = LirFunction {
            name,
            params,
            returns,
            block_ids: Vec::new()
        };

        self.program.functions.push(func);
        let func_id = LirFunctionId((self.program.functions.len() - 1) as u32);

        func_id
    }

    pub fn start_block(&mut self, fn_id: LirFunctionId, params: Vec<(Location, LirType)>) -> LirBlockId {
        let block = LirBlock {
            nodes: Vec::new(),
            params,
            term: None
        };

        self.program.blocks.push(block);
        let block_id = LirBlockId((self.program.blocks.len() - 1) as u32);

        self.program.functions[fn_id.0 as usize].block_ids.push(block_id);
        block_id
    }

    pub fn switch_block(&mut self, block_id: LirBlockId) {
        self.current_block = Some(block_id);
    }

    pub fn switch_function(&mut self, fn_id: LirFunctionId) {
        if let Some(block_id) = self.program.functions[fn_id.0 as usize].block_ids.first() {
            self.current_block = Some(*block_id);
        } else {
            self.current_block = None;
        }
    }

    pub fn get_block(&self, block_id: LirBlockId) -> Option<&LirBlock> {
        self.program.blocks.get(block_id.0 as usize)
    }

    pub fn get_function(&self, fn_id: LirFunctionId) -> Option<&LirFunction> {
        self.program.functions.get(fn_id.0 as usize)
    }

    pub fn get_program(&self) -> &LirProgram {
        &self.program
    }

    fn push_node(&mut self, node: LirNode) -> LirNodeId {
        self.program.nodes.push(node);
        let node_id = LirNodeId((self.program.nodes.len() - 1) as u32);

        if let Some(block_id) = self.current_block {
            self.program.blocks[block_id.0 as usize].nodes.push(node_id);
        }

        node_id
    }

    fn set_term(&mut self, term: LirTerminator) {
        if let Some(block_id) = self.current_block {
            self.program.blocks[block_id.0 as usize].term = Some(term);
        }
    }



    pub fn const_i64(&mut self, dst: Location, value: i64) -> LirNodeId {
        let node = LirNode::ConstI64 { dst, value };
        self.push_node(node)
    }

    pub fn const_f64(&mut self, dst: Location, value: f64) -> LirNodeId {
        let node = LirNode::ConstF64 { dst, value };
        self.push_node(node)
    }

    pub fn assign(&mut self, dst: Location, src: Location) -> LirNodeId {
        let node = LirNode::Assign { dst, src };
        self.push_node(node)
    }

    pub fn add(&mut self, dst: Location, lhs: Location, rhs: Location) -> LirNodeId {
        let node = LirNode::Add { dst, lhs, rhs };
        self.push_node(node)
    }

    pub fn return_(&mut self, src: Location) {
        let term = LirTerminator::Return { src };
        self.set_term(term);
    }

    pub fn call(&mut self, dst: Location, callee: Location, args: Vec<Location>) -> LirNodeId {
        let node = LirNode::Call { dst, callee, args };
        self.push_node(node)
    }

    pub fn load_builtin(&mut self, dst: Location, id: u32) -> LirNodeId {
        let node = LirNode::LoadBuiltin { dst, id };
        self.push_node(node)
    }
}
