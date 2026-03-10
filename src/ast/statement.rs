use crate::analysis::symbol_table::TypeSymTable;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::{Bool, StructType};
use crate::ast::expression::{Expression, UntypedExpr};
use crate::ast::statement::Statement::{BlockStmt, CommentStmt, ExpressionStmt, FunctionStmt, IfStmt, MultilineCommentStmt, ReturnStmt, StructStmt, VarDeclarationStmt, WhileStmt};
use crate::error::analysis_error::AnalysisError;
use crate::error::runtime_error::ValueTypeVariant;
use std::fmt::Debug;
use crate::ast::expression;
use crate::error::context::ErrorContext;
use crate::util::spanned::Spanned;

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
