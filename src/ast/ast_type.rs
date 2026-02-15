use std::fmt::Debug;
use tropaion_derive::ast_type;

pub trait AstType : Debug {
}


#[ast_type]
pub struct SymbolType(pub String);

#[ast_type]
pub struct ArrayType {
    pub underlying: Box<dyn AstType>,
    pub count: u32
}
