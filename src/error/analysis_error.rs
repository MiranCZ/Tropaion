use thiserror::Error;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::expression::UntypedExpr;
use crate::ast::statement::{Statement, UntypedStmt};
use crate::error::analysis_error::AnalysisError::{IllegalBinaryExpression, IllegalCall, IllegalIndexing, IllegalNullDeref, IllegalTypeAssignment, TypeMismatch};
use crate::error::Error;
use crate::error::runtime_error::ValueTypeVariant;
use crate::lexer::token::SimpleToken;

pub type EmptyRes = Result<(), AnalysisError>;


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum StatementType {
    Block, Function,
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum AnalysisError {
    #[error("Unresolved symbol '{0}'")]
    UnknownType(String),

    #[error("Illegal statement placed inside struct {0:?}")]
    IllegalStatementInStruct(UntypedStmt),

    #[error("Statement {0:?} was found without scope which is not allowed")]
    IllegalScopelessStatement(UntypedStmt),

    #[error("Expected statement to be {expected:?}, got {got:?} instead")]
    StatementMismatch {
        expected: StatementType,
        got: UntypedStmt
    },

    #[error("Expected variable '{0}' to be a constant")]
    ExpectedConst(String),

    #[error("Expected type to be {expected:?} got {got} instead")]
    TypeMismatch {
        expected: ValueTypeVariant,
        got: String
    },

    #[error("Type '{expected_type}' cannot be assigned to type '{tried}'")]
    IllegalTypeAssignment {
        expected_type: String,
        tried: String
    },

    #[error("Illegal binary expression '{left} {op_form} {right}'", op_form = op.string_representation())]
    IllegalBinaryExpression {
        left: String,
        op: SimpleToken,
        right: String,
    },

    #[error("Type '{called_type}' cannot be called")]
    IllegalCall {
        called_type: String
    },

    #[error("Type '{typ}' cannot be indexed")]
    IllegalIndexing {
        typ: String
    },

    #[error("Invalid place for a return")]
    DanglingReturn,

    #[error("Tried applying the '!!' operator to type {0} which is not a nullable type")]
    IllegalNullDeref(String),

    #[error("Cannot make a single type nullable multiple times")]
    RedundantNullable,

    #[error("Failed to resolve type {0}")]
    TypeResolutionFailed(String),

    #[error("Nullable values cannot be directly accessed with `.`")]
    NullableAccess,

    #[error("Constants must have an explicit type")]
    TypelessConst
}

impl Error for AnalysisError {
}

impl AnalysisError {

    pub fn type_mismatch(expected: ValueTypeVariant, got: TypeEntry, registry: &TypeRegistry) -> AnalysisError {
        TypeMismatch {expected, got: got.get(registry).format(registry)}
    }

    pub fn illegal_type_assignment(expected_type: TypeEntry, tried: TypeEntry, registry: &TypeRegistry) -> AnalysisError {
        IllegalTypeAssignment {
            expected_type: expected_type.get(registry).format(registry),
            tried: tried.get(registry).format(registry)
        }
    }

    pub fn illegal_binary_expression(left: TypeEntry,op: SimpleToken ,right: TypeEntry, registry: &TypeRegistry) -> AnalysisError {
        IllegalBinaryExpression {
            left: left.get(registry).format(registry),
            op,
            right: right.get(registry).format(registry)
        }
    }

    pub fn illegal_call(called: TypeEntry, registry: &TypeRegistry) -> AnalysisError {
        IllegalCall {
            called_type: called.get(registry).format(registry)
        }
    }

    pub fn illegal_indexing(indexed: TypeEntry, registry: &TypeRegistry) -> AnalysisError {
        IllegalIndexing {
            typ: indexed.get(registry).format(registry)
        }
    }

    pub fn illegal_null_deref(typ: TypeEntry, registry: &TypeRegistry) -> AnalysisError {
        IllegalNullDeref(typ.get(registry).format(registry))
    }

}