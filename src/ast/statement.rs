use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::expression::{Expression, TypedExpr, UntypedExpr};
use crate::ast::statement::Statement::ExpressionStmt;
use crate::error::context::Span;
use crate::util::spanned::Spanned;
use std::fmt::Debug;

pub type UntypedStmt = Spanned<Statement<()>>;
pub type TypedStmt = Spanned<Statement<TypeEntry>>;

pub type StatementBlock<T> = Vec<Spanned<Statement<T>>>;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<T> {
    BlockStmt {
        body: StatementBlock<T>
    },
    ExpressionStmt(Spanned<Expression<T>>),
    VarDeclarationStmt {
        name: String,
        is_const: bool,
        value: Spanned<Expression<T>>,
        explicit_type: Option<TypeEntry>
    },
    IfStmt {
        condition: Spanned<Expression<T>>,
        body: StatementBlock<T>,
        // either another `if_stmt` or `block_stmt`
        else_branch: Option<Box<Spanned<Statement<T>>>>
    },
    WhileStmt {
        condition: Spanned<Expression<T>>,
        body: StatementBlock<T>,
    },
    FunctionStmt {
        name: String,
        params: Vec<Parameter>,
        return_type: TypeEntry,
        body: StatementBlock<T>
    },
    StructStmt {
        name: String,
        fields: Vec<Parameter>,
        body: StatementBlock<T>
    },
    ReturnStmt(Spanned<Expression<T>>),
    CommentStmt(String),
    MultilineCommentStmt(String)
}

impl <T> Statement<T> {

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: TypeEntry
}

impl TypedStmt {

    pub fn err(registry: &mut TypeRegistry, span: Span) -> Statement<TypeEntry> {
        ExpressionStmt(Spanned::of(TypedExpr::err(registry), span))
    }

    pub fn is_err(&self, registry: &TypeRegistry) -> bool {
        if let ExpressionStmt(expr) = &self.node {
            return expr.is_err(registry);
        }

        false
    }

}

impl UntypedStmt {

    pub fn err(span: Span) -> Spanned<Statement<()>> {
        Spanned::of(ExpressionStmt(UntypedExpr::err(span)), span)
    }

}
