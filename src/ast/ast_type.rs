use std::collections::HashMap;
use std::fmt::{format, Debug};
use std::mem::swap;
use crate::analysis::symbol_table::TypeSymTable;
use crate::ast::ast_type::AstType::{ArrayType, FunctionType, FunctionsType, NullableType, ReferenceType, StructType, TupleType};
use crate::ast::statement::TypedStmt;

#[derive(Debug, PartialEq, Clone)]
pub enum AstType {
    UnknownType,
    Void,
    Bool,
    Int,
    Float,
    StringType,
    SymbolType(String),
    ReferenceType {
        underlying: Box<AstType>
    },
    NullableType {
        underlying: Box<AstType>
    },
    ArrayType {
        underlying: Box<AstType>,
        count: u32,
    },
    TupleType(Vec<AstType>),
    FunctionsType {
        name: String,
        overloads: Vec<AstType> // these should be only function types
    },
    FunctionType {
        name: String,
        params: Vec<AstType>,
        return_type: Box<AstType>
    },
    StructType {
        name: String,
        fields: Vec<MemberInfo>, 
        // fields and methods
        children: HashMap<String, MemberInfo>,
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct MemberInfo(pub AstType, pub String, pub u16);

impl AstType {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl AstType {

    pub fn try_assign(&self, other: Self) -> Option<Self> {
        match (self, other) {
            (AstType::Float, AstType::Int) => {
                Some(AstType::Float)
            },
            (NullableType {underlying}, AstType::NullableType {underlying: other_underlying}) => {
                if let AstType::UnknownType = **underlying {
                    Some(AstType::NullableType {underlying: other_underlying})
                } else {
                    if let AstType::UnknownType = *other_underlying {
                        Some(self.clone())
                    } else {
                        None
                    }
                }
            },
            (NullableType {underlying}, other) => {
                if let AstType::UnknownType = **underlying {
                    Some(NullableType {underlying: other.boxed()})
                } else {
                    None
                }
            }

            _ => None
        }
    }

}

impl AstType {
    
    pub fn resolve_type(self, symbol_table: &mut TypeSymTable) -> AstType {
        match self {
            AstType::SymbolType(name) => {
                let opt = symbol_table.get(name.clone());

                if let Some(t) = opt {
                    return t;
                }
                panic!("Failed to resolve symbol {name}")
            }
            ReferenceType {underlying, .. } => {
                let resolved = underlying.resolve_type(symbol_table);

                ReferenceType {underlying: resolved.boxed()}
            }
            ArrayType {underlying, count } => {
                let resolved = underlying.resolve_type(symbol_table);

                ArrayType {underlying: resolved.boxed(), count}
            }
            TupleType(arr) => {
                let mut resolved = vec![];

                for a in arr {
                    resolved.push(a.resolve_type(symbol_table));
                }

                TupleType(resolved)
            }
            FunctionsType { name, overloads } => {
                let mut resolved = vec![];

                for a in overloads {
                    resolved.push(a.resolve_type(symbol_table));
                }

                FunctionsType {name, overloads: resolved}
            }
            FunctionType { name, params, return_type } => {
                let return_type = return_type.resolve_type(symbol_table).boxed();

                let mut resolved = vec![];

                for a in params {
                    resolved.push(a.resolve_type(symbol_table));
                }

                FunctionType {name, params: resolved, return_type}
            }
            StructType {name, fields, children} => {

                println!("CALLED {name:?} {fields:?} {children:?}");
                let mut resolved_fields = vec![];

                for f in fields {
                    resolved_fields.push(MemberInfo(f.0.resolve_type(symbol_table), f.1, f.2));
                }

                let mut resolved_children = HashMap::new();

                for e in children {
                    let name = e.0;
                    let mem = e.1;

                    resolved_children.insert(name, MemberInfo(mem.0.resolve_type(symbol_table), mem.1, mem.2));
                }

                StructType {name, fields: resolved_fields, children: resolved_children}
            }


            _ => self
        }
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
            AstType::StructType {name, .. } => format!("L{name};"),
            _ => panic!()
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
            AstType::StructType {.. } => 1,
            _ => panic!()
        }
    }
    
}
