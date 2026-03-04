use thiserror::Error;
use crate::analysis::type_registry::TypeRegistry;
use crate::ast::ast_type::AstType;
use crate::compiler::bytecode::ByteCode;

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
    MissingVariable(String)
}

impl CompilationError {
    
    pub fn unsupported_type(typ: AstType, registry: &TypeRegistry) -> CompilationError {
        CompilationError::UnsupportedType(typ.format(registry))
    }
    
}