use std::collections::HashMap;
use std::fmt::Debug;
use std::mem::swap;

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
        fields: Vec<AstType>, 
        // fields and methods
        children: HashMap<String, AstType>,
    }
}

impl AstType {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl AstType {
   
    pub fn word_size(&self) -> u32 {
        match self {
            AstType::Void => 0,
            AstType::Bool | AstType::Int | AstType::Float | AstType::StringType | AstType::ReferenceType {..} => 1,
            AstType::SymbolType(_) => {
                panic!("Size not known for unresolved symbol {self:?}")  
            },
            AstType::ArrayType {count, underlying} => {
                underlying.word_size() * *count
            },
            AstType::TupleType(types) => {
                let mut size = 0;

                for x in types {
                    size += x.word_size();
                }
                
                size
            }
            AstType::FunctionType { .. } => 1,
            AstType::StructType {fields, .. } => {
                let mut size = 0;

                for x in fields {
                    size += x.word_size();
                }

                size
            }
        }
    }
    
}
