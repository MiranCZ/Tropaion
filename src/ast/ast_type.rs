use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone)]
pub enum AstType{
    Void,
    Bool,
    Int,
    Float,
    StringType,
    SymbolType(String),
    ReferenceType {
        underlying: Box<AstType>
    },
    ArrayType {
        underlying: Box<AstType>,
        count: u32,
    },
    TupleType(Vec<AstType>),
    Function {
        name: String,
        params: Vec<AstType>,
        return_type: Box<AstType>
    },
    Struct {
        name: String,
        fields: Vec<AstType>
    }
}

impl AstType {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
