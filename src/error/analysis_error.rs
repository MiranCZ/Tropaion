use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::statement::{Parameter, UntypedStmt};
use crate::error::analysis_error::AnalysisError::{FunctionAlreadyDefined, IllegalBinaryExpression, IllegalCall, IllegalFuncArgs, IllegalIndexing, IllegalMemberAccess, IllegalNullDeref, IllegalTypeAssignment, NameAlreadyUsed, TypeMismatch};
use crate::error::runtime_error::ValueTypeVariant;
use crate::lexer::token::SimpleToken;
use thiserror::Error;
use crate::ast::walking::folder::FoldedExpr;
use crate::util::spanned::Spanned;

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

    #[error("Could not find function '{func_name}' with parameters '({args})'")]
    IllegalFuncArgs {
        func_name: String,
        args: String
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
    TypelessConst,

    #[error("Tried accessing a member for type {0} which is not allowed")]
    IllegalMemberAccess(String),

    #[error("Tuples can only be indexed with non-negative int literals")]
    IllegalTupleIndex,

    #[error("Index is out of bounds (must satisfy 0 <= {0} < {1})")]
    IndexOutOfBounds(i64, i64),
    
    #[error("The name '{0}' is already in use")]
    NameAlreadyUsed(String),
    
    #[error("Function '{0}' is already defined")]
    FunctionAlreadyDefined(String)

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

    pub fn illegal_member_access(typ: TypeEntry, registry: &TypeRegistry) -> AnalysisError {
        IllegalMemberAccess(typ.format(registry))
    }

    pub fn illegal_func_args(name: String, resolved_args: Vec<Spanned<FoldedExpr<TypeEntry>>>, registry: &TypeRegistry) -> AnalysisError {

        if resolved_args.is_empty() {
            return IllegalFuncArgs {func_name: name, args: String::new()};
        }
        if resolved_args.len() == 1 {
            return IllegalFuncArgs {func_name: name, args: resolved_args[0].get_type().format(registry)};
        }


        let mut args_string = String::new();

        let mut iter = resolved_args.iter();

        args_string.push_str(iter.next().unwrap().get_type().format(registry).as_str());

        for arg in iter {
            args_string.push_str(", ");

            args_string.push_str(arg.get_type().format(registry).as_str());
        }

        IllegalFuncArgs {func_name: name, args: args_string}
    }
    
    pub fn function_already_defined(name: String, params: &Vec<Parameter>, registry: &TypeRegistry) -> AnalysisError {
        let mut str = name;

        str.push('(');
        
        
        if params.is_empty() {
            str.push(')');
            return FunctionAlreadyDefined(str);
        }
        if params.len() == 1 {
            str.push_str(params[0].param_type.format(registry).as_str());
            str.push(')');
            return FunctionAlreadyDefined(str);
        }

        let mut iter = params.iter();
        str.push_str(iter.next().unwrap().param_type.format(registry).as_str());
        
        for x in iter {
            str.push_str(", ");


            str.push_str(x.param_type.format(registry).as_str());
        }

        str.push(')');
        FunctionAlreadyDefined(str)
    }

}