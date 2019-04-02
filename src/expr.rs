use super::token::{self, Token};

pub enum Expr {
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
}

impl Expr {
    pub fn make_binary(left: Expr, operator: Token, right: Expr) -> Expr {
        Expr::Binary(BinaryExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    pub fn make_grouping(expression: Expr) -> Expr {
        Expr::Grouping(GroupingExpr {
            expression: Box::new(expression),
        })
    }

    pub fn make_literal(value: token::Literal) -> Expr {
        Expr::Literal(LiteralExpr { value })
    }

    pub fn make_unary(operator: Token, expression: Expr) -> Expr {
        Expr::Unary(UnaryExpr {
            operator,
            expression: Box::new(expression),
        })
    }
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
        visitor.visit_binary_expr(self)
    }
}

impl AcceptsVisitor for GroupingExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_grouping_expr(self)
    }
}
