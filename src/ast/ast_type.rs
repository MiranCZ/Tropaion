use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone)]
pub enum AstType {
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
    FunctionType {
        name: String,
        params: Vec<AstType>,
        return_type: Box<AstType>
    },
    StructType {
        name: String,
        
        // fields and methods
        children: HashMap<String, AstType>,
    }
}

impl AstType {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
