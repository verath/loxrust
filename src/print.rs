use super::expr::*;

pub struct AstPrinter {}

impl AstPrinter {
    pub fn print(&mut self, expr: Expr) -> String {
        expr.accept(self)
    }
}

impl Visitor for AstPrinter {
    type Result = String;

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> String {
        format!("{}{}", expr.left.accept(self), expr.right.accept(self))
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> String {
        format!("{}", expr.accept(self))
    }

    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> String {
        format!("{}", expr.value)
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> String {
        expr.expression.accept(self)
    }
}
