use crate::analysis::type_registry::TypeEntry;
use crate::ast::expression::Expression;
use crate::ast::statement::{Parameter, Statement, StatementBlock};
use crate::error::context::Span;
use crate::lexer::token::SimpleToken;
use crate::util::spanned::Spanned;

/// Transforms the typed AST by consuming and reconstructing nodes.
///
/// The walker (`walk_fold_*`) recurses into children first (bottom-up), then
/// calls the corresponding hook with the already-folded children. Override only
/// the hooks you care about; all defaults are identity (reconstruct unchanged).
///
/// To stop recursion into a subtree, override `fold_expr` / `fold_stmt` directly
/// and skip the `self.walk_fold_*` call.
pub trait Folder<I, O> where Self: Sized {

    // Top-level entry points (override to intercept before recursion)

    fn fold_stmt(&mut self, stmt: Spanned<Statement<I>>) -> Spanned<Statement<O>> {
        self.walk_fold_stmt(stmt)
    }

    fn fold_expr(&mut self, expr: Spanned<Expression<I>>) -> Spanned<Expression<O>> {
        self.walk_fold_expr(expr)
    }

    fn fold_type(&mut self, typ: I) -> O;

    // Statement hooks (children are already folded when these are called)

    fn fold_block(&mut self, body: StatementBlock<O>, span: Span) -> Spanned<Statement<O>> {
        Spanned::of(Statement::BlockStmt { body }, span)
    }

    fn fold_expression_stmt(&mut self, expr: Spanned<Expression<O>>, span: Span) -> Spanned<Statement<O>> {
        Spanned::of(Statement::ExpressionStmt(expr), span)
    }

    fn fold_var_declaration(&mut self, name: String, is_const: bool, value: Spanned<Expression<O>>, explicit_type: Option<TypeEntry>, span: Span) -> Spanned<Statement<O>> {
        Spanned::of(Statement::VarDeclarationStmt { name, is_const, value, explicit_type }, span)
    }

    fn fold_if(&mut self, condition: Spanned<Expression<O>>, body: StatementBlock<O>, else_branch: Option<Box<Spanned<Statement<O>>>>, span: Span) -> Spanned<Statement<O>> {
        Spanned::of(Statement::IfStmt { condition, body, else_branch }, span)
    }

    fn fold_while(&mut self, condition: Spanned<Expression<O>>, body: StatementBlock<O>, span: Span) -> Spanned<Statement<O>> {
        Spanned::of(Statement::WhileStmt { condition, body }, span)
    }

    fn fold_function(&mut self, name: String, params: Vec<Parameter>, return_type: TypeEntry, body: StatementBlock<O>, span: Span) -> Spanned<Statement<O>> {
        Spanned::of(Statement::FunctionStmt { name, params, return_type, body }, span)
    }

    fn fold_struct(&mut self, name: String, fields: Vec<Parameter>, body: StatementBlock<O>, span: Span) -> Spanned<Statement<O>> {
        Spanned::of(Statement::StructStmt { name, fields, body }, span)
    }

    fn fold_return(&mut self, expr: Spanned<Expression<O>>, span: Span) -> Spanned<Statement<O>> {
        Spanned::of(Statement::ReturnStmt(expr), span)
    }

    fn fold_comment(&mut self, s: String, span: Span) -> Spanned<Statement<O>> {
        Spanned::of(Statement::CommentStmt(s), span)
    }

    fn fold_multiline_comment(&mut self, s: String, span: Span) -> Spanned<Statement<O>> {
        Spanned::of(Statement::MultilineCommentStmt(s), span)
    }

    // Expression hooks (children are already folded when these are called)

    fn fold_null_literal(&mut self, t: O, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::NullLiteralExpr(t), span)
    }

    fn fold_bool_literal(&mut self, t: O, value: bool, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::BoolLiteralExpr(t, value), span)
    }

    fn fold_int_literal(&mut self, t: O, value: i64, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::IntLiteralExpr(t, value), span)
    }

    fn fold_float_literal(&mut self, t: O, value: f32, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::FloatLiteralExpr(t, value), span)
    }

    fn fold_string_literal(&mut self, t: O, value: String, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::StringLiteralExpr(t, value), span)
    }

    fn fold_identifier(&mut self, t: O, name: String, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::IdentifierExpr(t, name), span)
    }

    fn fold_array_literal(&mut self, t: O, values: Vec<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::ArrayLiteralExpr(t, values), span)
    }

    fn fold_nullable_expr(&mut self, t: O, inner: Box<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::NullableExpr(t, inner), span)
    }

    fn fold_increment(&mut self, t: O, expr: Box<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::IncrementExpr(t, expr), span)
    }

    fn fold_decrement(&mut self, t: O, expr: Box<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::DecrementExpr(t, expr), span)
    }

    fn fold_null_deref(&mut self, t: O, expr: Box<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::NullDerefExpr(t, expr), span)
    }

    fn fold_prefix(&mut self, t: O, operator: SimpleToken, expr: Box<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::PrefixExpr { t, operator, expr }, span)
    }

    fn fold_binary(&mut self, t: O, left: Box<Spanned<Expression<O>>>, operator: SimpleToken, right: Box<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::BinaryExpr { t, left, operator, right }, span)
    }

    fn fold_assign(&mut self, t: O, assignee: Box<Spanned<Expression<O>>>, value: Box<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::AssignExpr { t, assignee, value }, span)
    }

    fn fold_tuple(&mut self, t: O, values: Vec<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::TupleExpr { t, values }, span)
    }

    fn fold_array_access(&mut self, t: O, property: Box<Spanned<Expression<O>>>, index: Box<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::ArrayAccessExpr { t, property, index }, span)
    }

    fn fold_member(&mut self, t: O, member: Box<Spanned<Expression<O>>>, property: Box<Spanned<Expression<O>>>, null_safe: bool, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::MemberExpr { t, member, property, null_safe }, span)
    }

    fn fold_call(&mut self, t: O, func: Box<Spanned<Expression<O>>>, args: Vec<Spanned<Expression<O>>>, span: Span) -> Spanned<Expression<O>> {
        Spanned::of(Expression::CallExpr { t, func, args }, span)
    }

    // --- Walkers: recurse then dispatch. Not intended to be overridden. ---

    fn walk_fold_stmt(&mut self, stmt: Spanned<Statement<I>>) -> Spanned<Statement<O>> {
        let span = stmt.span;
        match stmt.node {
            Statement::BlockStmt { body } => {
                let body = body.into_iter().map(|s| self.fold_stmt(s)).collect();
                self.fold_block(body, span)
            }
            Statement::ExpressionStmt(expr) => {
                let expr = self.fold_expr(expr);
                self.fold_expression_stmt(expr, span)
            }
            Statement::VarDeclarationStmt { name, is_const, value, explicit_type } => {
                let value = self.fold_expr(value);
                self.fold_var_declaration(name, is_const, value, explicit_type, span)
            }
            Statement::IfStmt { condition, body, else_branch } => {
                let condition = self.fold_expr(condition);
                let body = body.into_iter().map(|s| self.fold_stmt(s)).collect();
                let else_branch = else_branch.map(|b| Box::new(self.fold_stmt(*b)));
                self.fold_if(condition, body, else_branch, span)
            }
            Statement::WhileStmt { condition, body } => {
                let condition = self.fold_expr(condition);
                let body = body.into_iter().map(|s| self.fold_stmt(s)).collect();
                self.fold_while(condition, body, span)
            }
            Statement::FunctionStmt { name, params, return_type, body } => {
                let body = body.into_iter().map(|s| self.fold_stmt(s)).collect();
                self.fold_function(name, params, return_type, body, span)
            }
            Statement::StructStmt { name, fields, body } => {
                let body = body.into_iter().map(|s| self.fold_stmt(s)).collect();
                self.fold_struct(name, fields, body, span)
            }
            Statement::ReturnStmt(expr) => {
                let expr = self.fold_expr(expr);
                self.fold_return(expr, span)
            }
            Statement::CommentStmt(s) => self.fold_comment(s, span),
            Statement::MultilineCommentStmt(s) => self.fold_multiline_comment(s, span),
        }
    }

    fn walk_fold_expr(&mut self, expr: Spanned<Expression<I>>) -> Spanned<Expression<O>> {
        let span = expr.span;
        match expr.node {
            Expression::NullLiteralExpr(t) => {
                let t= self.fold_type(t);
                self.fold_null_literal(t, span)
            }
            Expression::BoolLiteralExpr(t, v) => {
                let t= self.fold_type(t);
                self.fold_bool_literal(t, v, span)
            }
            Expression::IntLiteralExpr(t, v) => {
                let t= self.fold_type(t);

                self.fold_int_literal(t, v, span)
            }
            Expression::FloatLiteralExpr(t, v) => {
                let t= self.fold_type(t);
                self.fold_float_literal(t, v, span)
            }
            Expression::StringLiteralExpr(t, v) => {
                let t= self.fold_type(t);
                self.fold_string_literal(t, v, span)
            }
            Expression::IdentifierExpr(t, name) => {
                let t= self.fold_type(t);
                self.fold_identifier(t, name, span)
            }
            Expression::ArrayLiteralExpr(t, values) => {
                let t= self.fold_type(t);
                let values = values.into_iter().map(|v| self.fold_expr(v)).collect();
                self.fold_array_literal(t, values, span)
            }
            Expression::NullableExpr(t, inner) => {
                let t= self.fold_type(t);
                let inner = self.fold_expr(*inner).boxed();
                self.fold_nullable_expr(t, inner, span)
            }
            Expression::IncrementExpr(t, expr) => {
                let t= self.fold_type(t);
                let expr = self.fold_expr(*expr).boxed();
                self.fold_increment(t, expr, span)
            }
            Expression::DecrementExpr(t, expr) => {
                let t= self.fold_type(t);
                let expr = self.fold_expr(*expr).boxed();
                self.fold_decrement(t, expr, span)
            }
            Expression::NullDerefExpr(t, expr) => {
                let t= self.fold_type(t);
                let expr = self.fold_expr(*expr).boxed();
                self.fold_null_deref(t, expr, span)
            }
            Expression::PrefixExpr { t, operator, expr } => {
                let t= self.fold_type(t);
                let expr = self.fold_expr(*expr).boxed();
                self.fold_prefix(t, operator, expr, span)
            }
            Expression::BinaryExpr { t, left, operator, right } => {
                let t= self.fold_type(t);
                let left = self.fold_expr(*left).boxed();
                let right = self.fold_expr(*right).boxed();
                self.fold_binary(t, left, operator, right, span)
            }
            Expression::AssignExpr { t, assignee, value } => {
                let t= self.fold_type(t);
                let assignee = self.fold_expr(*assignee).boxed();
                let value = self.fold_expr(*value).boxed();
                self.fold_assign(t, assignee, value, span)
            }
            Expression::TupleExpr { t, values } => {
                let t= self.fold_type(t);
                let values = values.into_iter().map(|v| self.fold_expr(v)).collect();
                self.fold_tuple(t, values, span)
            }
            Expression::ArrayAccessExpr { t, property, index } => {
                let t= self.fold_type(t);
                let property = self.fold_expr(*property).boxed();
                let index = self.fold_expr(*index).boxed();
                self.fold_array_access(t, property, index, span)
            }
            Expression::MemberExpr { t, member, property, null_safe } => {
                let t= self.fold_type(t);
                let member = self.fold_expr(*member).boxed();
                let property = self.fold_expr(*property).boxed();
                self.fold_member(t, member, property, null_safe, span)
            }
            Expression::CallExpr { t, func, args } => {
                let t= self.fold_type(t);
                let func = self.fold_expr(*func).boxed();
                let args = args.into_iter().map(|a| self.fold_expr(a)).collect();
                self.fold_call(t, func, args, span)
            }
        }
    }
}