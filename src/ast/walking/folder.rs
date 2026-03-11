use crate::analysis::type_registry::TypeEntry;
use crate::analysis::type_registry::TypeRegistry;
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::MemberInfo;
use crate::ast::expression::Expression;
use crate::ast::statement::Parameter;
use crate::ast::statement::Statement;
use crate::ast::statement::StatementBlock;
use crate::error::context::Span;
use crate::lexer::token::SimpleToken;
use crate::util::spanned::Spanned;
use std::collections::HashMap;

// ── Type aliases ─────────────────────────────────────────────────────────────

pub type FoldedStmt<O> = Statement<O>;
pub type FoldedExpr<O> = Expression<O>;
pub type FoldedBlock<O> = StatementBlock<O>;

// ── Folder trait ─────────────────────────────────────────────────────────────

/// A tree-transforming visitor that rewrites `Statement<I>` / `Expression<I>`
/// into `Statement<O>` / `Expression<O>`.
///
/// The only *required* method is [`Folder::fold_annotation`], which converts
/// the per-expression type annotation from `I` to `O`. Every other method has
/// a structurally-recursive default implementation that you can override for
/// specific nodes.
///
/// # Type parameters
/// * `I` – input type annotation (e.g. `()` for untyped, `TypeEntry` for typed)
/// * `O` – output type annotation
pub trait Folder<I, O>
where
    Self: Sized,
    I: Clone,
    O: Clone,
{
    fn get_registry(&self) -> &TypeRegistry;
    fn get_registry_mut(&mut self) -> &mut TypeRegistry;

    /// Convert one type annotation embedded in an expression node.
    /// This is the only method that *must* be implemented.
    fn fold_annotation(&mut self, t: I) -> O;

    /// Convert a `TypeEntry` that appears in a structural position
    /// (parameter types, explicit variable types, return types).
    /// Defaults to the identity – override when you also want to rewrite
    /// structural types.
    fn fold_type_entry(&mut self, t: TypeEntry) -> TypeEntry {
        // FIXME this could use the registry to walk
        t
    }

    // ── Statements ───────────────────────────────────────────────────────────

    fn fold_stmt(&mut self, stmt: Spanned<Statement<I>>) -> Spanned<FoldedStmt<O>> {
        stmt.walk_fold(self)
    }

    fn fold_block(&mut self, body: StatementBlock<I>) -> FoldedBlock<O> {
        body.into_iter().map(|s| self.fold_stmt(s)).collect()
    }

    fn fold_expression_stmt(&mut self, expr: Spanned<Expression<I>>, span: Span) -> FoldedStmt<O> {
        Statement::ExpressionStmt(self.fold_expr(expr))
    }

    fn fold_var_declaration(
        &mut self,
        name: String,
        is_const: bool,
        value: Spanned<Expression<I>>,
        explicit_type: Option<TypeEntry>,
        span: Span,
    ) -> FoldedStmt<O> {
        Statement::VarDeclarationStmt {
            name,
            is_const,
            value: self.fold_expr(value),
            explicit_type: explicit_type.map(|t| self.fold_type_entry(t)),
        }
    }

    fn fold_if(
        &mut self,
        condition: Spanned<Expression<I>>,
        body: StatementBlock<I>,
        else_branch: Option<Box<Spanned<Statement<I>>>>,
        span: Span,
    ) -> FoldedStmt<O> {
        Statement::IfStmt {
            condition: self.fold_expr(condition),
            body: self.fold_block(body),
            else_branch: else_branch.map(|b| self.fold_stmt(*b).boxed()),
        }
    }

    fn fold_while(
        &mut self,
        condition: Spanned<Expression<I>>,
        body: StatementBlock<I>,
        span: Span,
    ) -> FoldedStmt<O> {
        Statement::WhileStmt {
            condition: self.fold_expr(condition),
            body: self.fold_block(body),
        }
    }

    fn fold_function(
        &mut self,
        name: String,
        params: Vec<Parameter>,
        return_type: TypeEntry,
        body: StatementBlock<I>,
        span: Span,
    ) -> FoldedStmt<O> {
        let folded_params = params
            .into_iter()
            .map(|p| Parameter {
                name: p.name,
                param_type: self.fold_type_entry(p.param_type),
            })
            .collect();

        Statement::FunctionStmt {
            name,
            params: folded_params,
            return_type: self.fold_type_entry(return_type),
            body: self.fold_block(body),
        }
    }

    fn fold_struct(
        &mut self,
        name: String,
        fields: Vec<Parameter>,
        body: StatementBlock<I>,
        span: Span,
    ) -> FoldedStmt<O> {
        let folded_fields = fields
            .into_iter()
            .map(|p| Parameter {
                name: p.name,
                param_type: self.fold_type_entry(p.param_type),
            })
            .collect();

        Statement::StructStmt {
            name,
            fields: folded_fields,
            body: self.fold_block(body),
        }
    }

    fn fold_return(&mut self, expr: Spanned<Expression<I>>, span: Span) -> FoldedStmt<O> {
        Statement::ReturnStmt(self.fold_expr(expr))
    }

    fn fold_comment(&mut self, comment: String, span: Span) -> FoldedStmt<O> {
        Statement::CommentStmt(comment)
    }

    fn fold_multiline_comment(&mut self, comment: String, span: Span) -> FoldedStmt<O> {
        Statement::MultilineCommentStmt(comment)
    }

    // ── Expressions ──────────────────────────────────────────────────────────

    fn fold_expr(&mut self, expr: Spanned<Expression<I>>) -> Spanned<FoldedExpr<O>> {
        expr.walk_fold(self)
    }

    fn fold_errored(&mut self, t: I, span: Span) -> FoldedExpr<O> {
        Expression::ErroredExpr(self.fold_annotation(t))
    }

    fn fold_null_literal(&mut self, t: I, span: Span) -> FoldedExpr<O> {
        Expression::NullLiteralExpr(self.fold_annotation(t))
    }

    fn fold_bool_literal(&mut self, t: I, value: bool, span: Span) -> FoldedExpr<O> {
        Expression::BoolLiteralExpr(self.fold_annotation(t), value)
    }

    fn fold_int_literal(&mut self, t: I, value: i64, span: Span) -> FoldedExpr<O> {
        Expression::IntLiteralExpr(self.fold_annotation(t), value)
    }

    fn fold_float_literal(&mut self, t: I, value: f32, span: Span) -> FoldedExpr<O> {
        Expression::FloatLiteralExpr(self.fold_annotation(t), value)
    }

    fn fold_string_literal(&mut self, t: I, value: String, span: Span) -> FoldedExpr<O> {
        Expression::StringLiteralExpr(self.fold_annotation(t), value)
    }

    fn fold_identifier(&mut self, t: I, name: String, span: Span) -> FoldedExpr<O> {
        Expression::IdentifierExpr(self.fold_annotation(t), name)
    }

    fn fold_array_literal(
        &mut self,
        t: I,
        values: Vec<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        let folded_values = values.into_iter().map(|v| self.fold_expr(v)).collect();

        Expression::ArrayLiteralExpr(self.fold_annotation(t), folded_values)
    }

    fn fold_nullable_expr(
        &mut self,
        t: I,
        inner: Box<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        Expression::NullableExpr(self.fold_annotation(t), self.fold_expr(*inner).boxed())
    }

    fn fold_increment(
        &mut self,
        t: I,
        expr: Box<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        Expression::IncrementExpr(self.fold_annotation(t), self.fold_expr(*expr).boxed())
    }

    fn fold_decrement(
        &mut self,
        t: I,
        expr: Box<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        Expression::DecrementExpr(self.fold_annotation(t), self.fold_expr(*expr).boxed())
    }

    fn fold_null_deref(
        &mut self,
        t: I,
        expr: Box<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        Expression::NullDerefExpr(self.fold_annotation(t), self.fold_expr(*expr).boxed())
    }

    fn fold_prefix(
        &mut self,
        t: I,
        operator: SimpleToken,
        expr: Box<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        Expression::PrefixExpr {
            t: self.fold_annotation(t),
            operator,
            expr: self.fold_expr(*expr).boxed(),
        }
    }

    fn fold_binary(
        &mut self,
        t: I,
        left: Box<Spanned<Expression<I>>>,
        operator: SimpleToken,
        right: Box<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        Expression::BinaryExpr {
            t: self.fold_annotation(t),
            left: self.fold_expr(*left).boxed(),
            operator,
            right: self.fold_expr(*right).boxed(),
        }
    }

    fn fold_assign(
        &mut self,
        t: I,
        assignee: Box<Spanned<Expression<I>>>,
        value: Box<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        Expression::AssignExpr {
            t: self.fold_annotation(t),
            assignee: self.fold_expr(*assignee).boxed(),
            value: self.fold_expr(*value).boxed(),
        }
    }

    fn fold_tuple(
        &mut self,
        t: I,
        values: Vec<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        let folded_values = values.into_iter().map(|v| self.fold_expr(v)).collect();

        Expression::TupleExpr {
            t: self.fold_annotation(t),
            values: folded_values,
        }
    }

    fn fold_array_access(
        &mut self,
        t: I,
        property: Box<Spanned<Expression<I>>>,
        index: Box<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        Expression::ArrayAccessExpr {
            t: self.fold_annotation(t),
            property: self.fold_expr(*property).boxed(),
            index: self.fold_expr(*index).boxed(),
        }
    }

    fn fold_member(
        &mut self,
        t: I,
        member: Box<Spanned<Expression<I>>>,
        property: Box<Spanned<Expression<I>>>,
        null_safe: bool,
        span: Span,
    ) -> FoldedExpr<O> {
        Expression::MemberExpr {
            t: self.fold_annotation(t),
            member: self.fold_expr(*member).boxed(),
            property: self.fold_expr(*property).boxed(),
            null_safe,
        }
    }

    fn fold_call(
        &mut self,
        t: I,
        func: Box<Spanned<Expression<I>>>,
        args: Vec<Spanned<Expression<I>>>,
        span: Span,
    ) -> FoldedExpr<O> {
        let folded_args = args.into_iter().map(|a| self.fold_expr(a)).collect();

        Expression::CallExpr {
            t: self.fold_annotation(t),
            func: self.fold_expr(*func).boxed(),
            args: folded_args,
        }
    }

    // ── AstType / TypeEntry ──────────────────────────────────────────────────

    fn fold_ast_type(&mut self, typ: AstType) -> AstType {
        typ.walk_fold(self)
    }

    fn fold_unknown_type(&mut self) -> AstType {
        AstType::UnknownType
    }
    fn fold_void_type(&mut self) -> AstType {
        AstType::Void
    }
    fn fold_bool_type(&mut self) -> AstType {
        AstType::Bool
    }
    fn fold_int_type(&mut self) -> AstType {
        AstType::Int
    }
    fn fold_float_type(&mut self) -> AstType {
        AstType::Float
    }
    fn fold_string_type(&mut self) -> AstType {
        AstType::StringType
    }

    fn fold_symbol_type(&mut self, name: String) -> AstType {
        AstType::SymbolType(name)
    }

    fn fold_reference_type(&mut self, underlying: TypeEntry) -> AstType {
        AstType::ReferenceType {
            underlying: self.fold_type_entry(underlying),
        }
    }

    fn fold_nullable_type(&mut self, underlying: TypeEntry) -> AstType {
        AstType::NullableType {
            underlying: self.fold_type_entry(underlying),
        }
    }

    fn fold_array_type(&mut self, underlying: TypeEntry) -> AstType {
        AstType::ArrayType {
            underlying: self.fold_type_entry(underlying),
        }
    }

    fn fold_tuple_type(&mut self, types: Vec<TypeEntry>) -> AstType {
        AstType::TupleType(types.into_iter().map(|t| self.fold_type_entry(t)).collect())
    }

    fn fold_functions_type(&mut self, name: String, overloads: Vec<TypeEntry>) -> AstType {
        AstType::FunctionsType {
            name,
            overloads: overloads
                .into_iter()
                .map(|t| self.fold_type_entry(t))
                .collect(),
        }
    }

    fn fold_function_type(
        &mut self,
        name: String,
        params: Vec<TypeEntry>,
        return_type: TypeEntry,
    ) -> AstType {
        AstType::FunctionType {
            name,
            params: params
                .into_iter()
                .map(|t| self.fold_type_entry(t))
                .collect(),
            return_type: self.fold_type_entry(return_type),
        }
    }

    fn fold_struct_type(
        &mut self,
        name: String,
        fields: Vec<MemberInfo>,
        children: HashMap<String, MemberInfo>,
    ) -> AstType {
        let folded_fields = fields
            .into_iter()
            .map(|MemberInfo(t, n, idx)| MemberInfo(self.fold_type_entry(t), n, idx))
            .collect();

        let folded_children = children
            .into_iter()
            .map(|(k, MemberInfo(t, n, idx))| (k, MemberInfo(self.fold_type_entry(t), n, idx)))
            .collect();

        AstType::StructType {
            name,
            fields: folded_fields,
            children: folded_children,
        }
    }
}

// ── walk_fold implementations ─────────────────────────────────────────────────

impl<I: Clone> Spanned<Statement<I>> {
    pub fn walk_fold<O, F>(self, folder: &mut F) -> Spanned<Statement<O>>
    where
        O: Clone,
        F: Folder<I, O>,
    {
        let span = self.span;
        Spanned::of(
            match self.node {
                Statement::BlockStmt { body } => Statement::BlockStmt {
                    body: folder.fold_block(body),
                },
                Statement::ExpressionStmt(expr) => folder.fold_expression_stmt(expr, span),
                Statement::VarDeclarationStmt {
                    name,
                    is_const,
                    value,
                    explicit_type,
                } => folder.fold_var_declaration(name, is_const, value, explicit_type, span),
                Statement::IfStmt {
                    condition,
                    body,
                    else_branch,
                } => folder.fold_if(condition, body, else_branch, span),
                Statement::WhileStmt { condition, body } => {
                    folder.fold_while(condition, body, span)
                }
                Statement::FunctionStmt {
                    name,
                    params,
                    return_type,
                    body,
                } => folder.fold_function(name, params, return_type, body, span),
                Statement::StructStmt { name, fields, body } => {
                    folder.fold_struct(name, fields, body, span)
                }
                Statement::ReturnStmt(expr) => folder.fold_return(expr, span),
                Statement::CommentStmt(s) => folder.fold_comment(s, span),
                Statement::MultilineCommentStmt(s) => folder.fold_multiline_comment(s, span),
            },
            span,
        )
    }
}

impl<I: Clone> Spanned<Expression<I>> {
    pub fn walk_fold<O, F>(self, folder: &mut F) -> Spanned<Expression<O>>
    where
        O: Clone,
        F: Folder<I, O>,
    {
        let span = self.span;
        Spanned::of(
            match self.node {
                Expression::ErroredExpr(t) => folder.fold_errored(t, span),
                Expression::NullLiteralExpr(t) => folder.fold_null_literal(t, span),
                Expression::BoolLiteralExpr(t, v) => folder.fold_bool_literal(t, v, span),
                Expression::IntLiteralExpr(t, v) => folder.fold_int_literal(t, v, span),
                Expression::FloatLiteralExpr(t, v) => folder.fold_float_literal(t, v, span),
                Expression::StringLiteralExpr(t, v) => folder.fold_string_literal(t, v, span),
                Expression::IdentifierExpr(t, name) => folder.fold_identifier(t, name, span),
                Expression::ArrayLiteralExpr(t, values) => {
                    folder.fold_array_literal(t, values, span)
                }
                Expression::NullableExpr(t, inner) => folder.fold_nullable_expr(t, inner, span),
                Expression::IncrementExpr(t, expr) => folder.fold_increment(t, expr, span),
                Expression::DecrementExpr(t, expr) => folder.fold_decrement(t, expr, span),
                Expression::NullDerefExpr(t, expr) => folder.fold_null_deref(t, expr, span),
                Expression::PrefixExpr { t, operator, expr } => {
                    folder.fold_prefix(t, operator, expr, span)
                }
                Expression::BinaryExpr {
                    t,
                    left,
                    operator,
                    right,
                } => folder.fold_binary(t, left, operator, right, span),
                Expression::AssignExpr { t, assignee, value } => {
                    folder.fold_assign(t, assignee, value, span)
                }
                Expression::TupleExpr { t, values } => folder.fold_tuple(t, values, span),
                Expression::ArrayAccessExpr { t, property, index } => {
                    folder.fold_array_access(t, property, index, span)
                }
                Expression::MemberExpr {
                    t,
                    member,
                    property,
                    null_safe,
                } => folder.fold_member(t, member, property, null_safe, span),
                Expression::CallExpr { t, func, args } => folder.fold_call(t, func, args, span),
            },
            span,
        )
    }
}

impl AstType {
    pub fn walk_fold<I, O, F>(self, folder: &mut F) -> AstType
    where
        I: Clone,
        O: Clone,
        F: Folder<I, O>,
    {
        match self {
            // should not be able to fold errors
            AstType::ErroredType => self,

            AstType::UnknownType => folder.fold_unknown_type(),
            AstType::Void => folder.fold_void_type(),
            AstType::Bool => folder.fold_bool_type(),
            AstType::Int => folder.fold_int_type(),
            AstType::Float => folder.fold_float_type(),
            AstType::StringType => folder.fold_string_type(),
            AstType::SymbolType(name) => folder.fold_symbol_type(name),
            AstType::ReferenceType { underlying } => folder.fold_reference_type(underlying),
            AstType::NullableType { underlying } => folder.fold_nullable_type(underlying),
            AstType::ArrayType { underlying } => folder.fold_array_type(underlying),
            AstType::TupleType(types) => folder.fold_tuple_type(types),
            AstType::FunctionsType { name, overloads } => {
                folder.fold_functions_type(name, overloads)
            }
            AstType::FunctionType {
                name,
                params,
                return_type,
            } => folder.fold_function_type(name, params, return_type),
            AstType::StructType {
                name,
                fields,
                children,
            } => folder.fold_struct_type(name, fields, children),
        }
    }
}
