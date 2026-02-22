use std::collections::HashMap;
use std::fmt::{format, Debug};
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
    pub fn get_type_name(&self) -> String {
        match self {
            AstType::Void => "V".to_string(),
            AstType::Bool => "b".to_string(),
            AstType::Int => "i".to_string(),
            AstType::Float => "f".to_string(),
            AstType::StringType => "s".to_string(),
            AstType::SymbolType(n) => format!("L{n};"),
            AstType::ReferenceType { underlying } => underlying.get_type_name(), // references do not affect method signature
            AstType::ArrayType {underlying, .. } => format!("A{};",underlying.get_type_name()),
            AstType::TupleType(types) => {
                let mut name = "T".to_string();
                for t in types {
                    name += t.get_type_name().as_str();
                }

                name + ";"
            }
            AstType::FunctionType { .. } => panic!("Functions do not have names!"),
            AstType::StructType {name, .. } => format!("L{name};")
        }
    }

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
