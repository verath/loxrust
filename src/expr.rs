use super::token::{self, Token};

pub enum Expr {
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
}

pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

pub struct LiteralExpr {
    pub value: token::Literal,
}

pub struct UnaryExpr {
    pub operator: Token,
    pub expression: Box<Expr>,
}

pub trait Visitor {
    type Result;

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Self::Result;
    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Self::Result;
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Self::Result;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Self::Result;
}

pub trait AcceptsVisitor {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result;
}

impl AcceptsVisitor for Expr {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        use Expr::*;
        match *self {
            Binary(ref expr) => visitor.visit_binary_expr(expr),
            Grouping(ref expr) => visitor.visit_grouping_expr(expr),
            Literal(ref expr) => visitor.visit_literal_expr(expr),
            Unary(ref expr) => visitor.visit_unary_expr(expr),
        }
    }
}

impl AcceptsVisitor for BinaryExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        return visitor.visit_binary_expr(self);
    }
}

impl AcceptsVisitor for GroupingExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        return visitor.visit_grouping_expr(self);
    }
}
