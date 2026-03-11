use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::expression::Expression;
use crate::ast::expression::Expression::PrefixExpr;
use crate::ast::statement::TypedStmt;
use crate::ast::walking::folder::{FoldedExpr, Folder};
use crate::error::context::Span;
use crate::lexer::token::SimpleToken;
use crate::util::spanned::Spanned;

// TODO implement more folding cases to improve performance
pub struct ConstExprFolder<'a> {
    registry: &'a mut TypeRegistry,
}

impl <'a> ConstExprFolder<'a> {

    pub fn new(registry: &'a mut TypeRegistry) -> Self {
        Self {
            registry
        }
    }

    pub fn fold(stmt: TypedStmt, registry: &'a mut TypeRegistry) -> TypedStmt{
        let mut new = Self::new(registry);

        stmt.walk_fold(&mut new)
    }

}

impl<'a> Folder<TypeEntry, TypeEntry> for ConstExprFolder<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn fold_annotation(&mut self, t: TypeEntry) -> TypeEntry {
        t
    }

    fn fold_type_entry(&mut self, t: TypeEntry) -> TypeEntry {
        t
    }

    fn fold_prefix(&mut self, t: TypeEntry, operator: SimpleToken, expr: Box<Spanned<Expression<TypeEntry>>>, span: Span) -> FoldedExpr<TypeEntry> {
        let expr = self.fold_expr(*expr);

        match operator {
            SimpleToken::Dash => {
                if let Expression::IntLiteralExpr(t, i) = expr.node {
                    return Expression::IntLiteralExpr(t, -i);
                }
                if let Expression::FloatLiteralExpr(t, f) = expr.node {
                    return Expression::FloatLiteralExpr(t, -f);
                }
            }

            _ => {}
        };

        PrefixExpr {t, operator, expr: expr.boxed()}
    }
}
