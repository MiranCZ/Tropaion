use thiserror::Error;
use crate::analysis::type_registry::TypeRegistry;
use crate::ast::ast_type::AstType;
use crate::compiler::bytecode::ByteCode;
use crate::error::compilation_error::CompilationError::{IllegalCall, IllegalIndexing, IllegalMemberAccess};
use crate::error::runtime_error::ValueTypeVariant;
use crate::lexer::token::SimpleToken;

pub type EmptyRes = Result<(), CompilationError>;

#[derive(Error, Debug, PartialEq)]
pub enum CompilationError {

    #[error("Expected comparison placeholder, got {0:?}")]
    ExpectedComparison(ByteCode),

    #[error("Expected a scope but none was found")]
    MissingScope,

    #[error("Encountered unsupported type {0}")]
    UnsupportedType(String),

    #[error("Variable with the name {0} not created")]
    MissingVariable(String),

    #[error("Invalid operator {0:?}")]
    InvalidOperator(SimpleToken),

    #[error("Expected {expected:?} got {got} instead")]
    TypeMismatch{expected: ValueTypeVariant, got: String},

    #[error("Illegal binary operator {0:?}")]
    IllegalBinOperator(SimpleToken),

    #[error("Struct size is too big ({0} > {max})", max = u16::MAX)]
    StructTooLarge(u32),
   
    #[error("Type {0} cannot be called")]
    IllegalCall(String),

    #[error("Tried accessing a member for type {0} which is not allowed")]
    IllegalMemberAccess(String),
   
    #[error("Tried accessing missing member '{0}'")]
    MemberNotFound(String),
   
    #[error("Tried indexing type {0} which is not allowed")]
    IllegalIndexing(String),

    #[error("Int constant of '{0}' does not fit integer bounds ({min} < n < {max})", min=i32::MIN, max=i32::MAX)]
    IntOutOfBounds(i64)
}

impl CompilationError {

    pub fn unsupported_type(typ: AstType, registry: &TypeRegistry) -> CompilationError {
        panic!();
        CompilationError::UnsupportedType(typ.format(registry))
    }

    pub fn type_mismatch(expected: ValueTypeVariant,typ: AstType, registry: &TypeRegistry) -> CompilationError {
        CompilationError::TypeMismatch {expected, got: typ.format(registry)}
    }
    
    pub fn illegal_call(typ: AstType, registry: &TypeRegistry) -> CompilationError {
        IllegalCall(typ.format(registry))
    }
    
    pub fn illegal_member_access(typ: AstType, registry: &TypeRegistry) -> CompilationError {
        panic!();
        IllegalMemberAccess(typ.format(registry))
    }
    
    pub fn illegal_indexing(typ: AstType, registry: &TypeRegistry) -> CompilationError {
        IllegalIndexing(typ.format(registry))
    }

}