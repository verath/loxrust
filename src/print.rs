use super::expr::*;
use super::token;

pub struct AstPrinter {}

#[allow(clippy::new_without_default)]
impl AstPrinter {
    pub fn new() -> Self {
        AstPrinter {}
    }

    pub fn print(&mut self, expr: Expr) -> String {
        expr.accept(self)
    }
}

impl Visitor for AstPrinter {
    type Result = String;

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> String {
        format!(
            "({op} {left} {right})",
            op = expr.operator.lexeme,
            left = expr.left.accept(self),
            right = expr.right.accept(self)
        )
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> String {
        format!("(group {})", expr.expression.accept(self))
    }

    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> String {
        use token::Literal;
        match expr.value {
            Literal::Number(n) => format!("{}", n),
            Literal::String(ref s) => format!("\"{}\"", s),
        }
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> String {
        format!(
            "({op} {expr})",
            op = expr.operator.lexeme,
            expr = expr.expression.accept(self)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::{Literal, Token, TokenType};

    fn make_token(token_type: TokenType, lexeme: &str) -> Token {
        Token {
            token_type,
            lexeme: lexeme.to_owned(),
            line: 1,
            literal: None,
        }
    }

    #[test]
    fn test_visit_binary_expr() {
        let mut printer = AstPrinter::new();
        let ex = Expr::make_binary(
            Expr::make_literal(Literal::Number(0.0)),
            make_token(TokenType::Star, "*"),
            Expr::make_literal(Literal::Number(1.0)),
        );
        assert_eq!(ex.accept(&mut printer), "(* 0 1)");
    }

    #[test]
    fn test_visit_grouping_expr() {
        let mut printer = AstPrinter::new();
        let ex = Expr::make_grouping(Expr::make_literal(Literal::Number(102.02)));
        assert_eq!(ex.accept(&mut printer), "(group 102.02)");
    }

    #[test]
    fn test_visit_literal_expr() {
        let mut printer = AstPrinter::new();
        let ex = Expr::make_literal(Literal::Number(2.0));
        assert_eq!(ex.accept(&mut printer), "2");
        let ex = Expr::make_literal(Literal::String(String::from("2.0")));
        assert_eq!(ex.accept(&mut printer), "\"2.0\"");
    }

    #[test]
    fn test_visit_unary_expr() {
        let mut printer = AstPrinter::new();
        let ex = Expr::make_unary(
            make_token(TokenType::Minus, "-"),
            Expr::make_literal(Literal::Number(2.0)),
        );
        assert_eq!(ex.accept(&mut printer), "(- 2)");
    }

    #[test]
    fn test_print() {
        let mut printer = AstPrinter::new();

        let ex = Expr::make_binary(
            Expr::make_unary(
                make_token(TokenType::Minus, "-"),
                Expr::make_literal(Literal::Number(123.0)),
            ),
            make_token(TokenType::Star, "*"),
            Expr::make_grouping(Expr::make_literal(Literal::Number(45.67))),
        );

        let expected = "(* (- 123) (group 45.67))";
        assert_eq!(printer.print(ex), expected);
    }
}
