use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub enum AstType{
    SymbolType(String),
    ReferenceType {
        underlying: Box<AstType>
    },
    ArrayType {
        underlying: Box<AstType>,
        count: u32,
    },
    TupleType(Vec<AstType>)
}

impl AstType {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
