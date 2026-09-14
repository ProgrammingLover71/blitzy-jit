pub trait AstVisitor<R> {
    fn visit<R>(&mut self, node: &dyn AstNode) -> R;
}


pub enum AstNodeType {
    IntLiteral,

    ReturnStatement,
}


pub trait AstNode {
    fn node_type(&self) -> AstNodeType;
    fn accept<R>(&self, visitor: &mut dyn AstVisitor<R>) -> R;
}

