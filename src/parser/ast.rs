use std::fmt;
use crate::lexer::token;


// Represents a node ID in an arena
#[derive(Clone, Copy)]
pub struct NodeId(pub u32);

// The type of an AST node. Holds its data
pub enum AstNodeType {
    IntLiteral {
        value: i64
    },

    BinaryOp {
        left: NodeId,
        right: NodeId,
        op: OpType
    },

    Return {
        value: NodeId
    }
}

// The type of a binary operation
pub enum OpType {
    Add,
    Subtract,
    Multiply,
    Divide
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

// An AST program. Holds a copy of the parser's node arena and a vector of root node IDs
pub struct Program<'a> {
    pub arena: &'a mut NodeArena,
    pub roots: Vec<NodeId>
}


//====== Implementations ======//


impl<'a> Program<'a> {
    pub fn new(arena: &'a mut NodeArena, roots: Vec<NodeId>) -> Self {
        Self { arena, roots }
    }

    pub fn get_node(&self, id: NodeId) -> Option<&AstNode> {
        self.arena.get(id)
    }
}


impl AstNode {
    fn new(ln: u32, col: u32, typ: AstNodeType) -> Self {
        Self { n_line: ln, n_col: col, n_type: typ }
    }
}


impl OpType {
    pub fn from_token(t: token::TokenType) -> Self {
        match t {
            token::TokenType::Plus => OpType::Add,
            token::TokenType::Minus => OpType::Subtract,
            token::TokenType::Star => OpType::Multiply,
            token::TokenType::Slash => OpType::Divide,
            _ => unreachable!()
        }
    }
}


impl fmt::Display for OpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpType::Add => write!(f, "+"),
            OpType::Subtract => write!(f, "-"),
            OpType::Multiply => write!(f, "*"),
            OpType::Divide => write!(f, "/"),
        }
    }
}


impl fmt::Display for AstNodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNodeType::IntLiteral { 
                value
            } => write!(f, "Int({})", value),

            AstNodeType::BinaryOp { 
                left,
                right,
                op  
            } => write!(f, "BinaryOp(left: {}, right: {}, op: {})", left.0, right.0, op),

            AstNodeType::Return { 
                value
            } => write!(f, "Return(value: {})", value.0),
        }
    }
}


impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AstNode(line: {}, col: {}, type: {})", self.n_line, self.n_col, self.n_type)
    }
    
}


impl NodeArena {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn alloc(&mut self, node: AstNode) -> NodeId {
        self.nodes.push(node);
        NodeId((self.nodes.len() as u32) - 1)
    }

    pub fn get(&self, id: NodeId) -> Option<&AstNode> {
        self.nodes.get(id.0 as usize)
    }

    pub fn nodes(&self) -> &Vec<AstNode> {
        &self.nodes
    }
}
