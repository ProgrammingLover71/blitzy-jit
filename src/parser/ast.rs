// Represents a node ID in an arena
pub struct NodeId(u32);

// The type of an AST node. Holds its data
pub enum AstNodeType {
    IntLiteral {
        value: i64
    },

    BinaryOp {
        left: NodeId,
        right: NodeId,
    },

    Return {
        value: NodeId
    }
}

// An AST node. Holds line/column and data info
pub struct AstNode {
    pub n_line: u32,
    pub n_col: u32,
    pub n_type: AstNodeType
}

// A node arena. Holds a vector of nodes to keep locality
pub struct NodeArena {
    nodes: Vec<AstNode>
}


impl AstNode {
    fn new(ln: u32, col: u32, typ: AstNodeType) -> Self {
        Self { n_line: ln, n_col: col, n_type: typ }
    }
}


impl NodeArena {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn add(&mut self, node: AstNode) -> NodeId {
        self.nodes.push(node);
        NodeId((self.nodes.len() as u32) - 1)
    }

    pub fn get(&self, id: NodeId) -> Option<&AstNode> {
        self.nodes.get(id.0 as usize)
    }
}
