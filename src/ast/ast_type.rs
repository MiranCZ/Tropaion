use std::collections::HashMap;
use std::fmt::{format, Debug};
use std::mem::swap;
use crate::analysis::symbol_table::TypeSymTable;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::{ArrayType, FunctionType, FunctionsType, NullableType, ReferenceType, StructType, TupleType};
use crate::ast::statement::TypedStmt;
use crate::error::analysis_error::AnalysisError;
use crate::error::analysis_error::AnalysisError::{RedundantNullable, ResolutionFailed};
use crate::error::context::ErrorContext;

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
        underlying: TypeEntry
    },
    NullableType {
        underlying: TypeEntry
    },
    ArrayType {
        underlying: TypeEntry
    },
    TupleType(Vec<TypeEntry>),
    FunctionsType {
        name: String,
        overloads: Vec<TypeEntry> // these should be only function types
    },
    FunctionType {
        name: String,
        params: Vec<TypeEntry>,
        return_type: TypeEntry
    },
    StructType {
        name: String,
        fields: Vec<MemberInfo>, 
        // fields and methods
        children: HashMap<String, MemberInfo>,
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct MemberInfo(pub TypeEntry, pub String, pub u16);

impl AstType {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl AstType {

    pub fn format(&self, registry: &TypeRegistry) -> String {
        match self {
            AstType::UnknownType => "<unknown>".to_string(),
            AstType::Void => "void".to_string(),
            AstType::Bool => "bool".to_string(),
            AstType::Int => "int".to_string(),
            AstType::Float => "float".to_string(),
            AstType::StringType => "string".to_string(),
            AstType::SymbolType(s) => format!("<symbol({s})>"),

            AstType::ReferenceType { underlying } => format!("&{}", underlying.get(registry).format(registry)),
            AstType::NullableType { underlying } => format!("{}?", underlying.get(registry).format(registry)),
            AstType::ArrayType { underlying } => format!("[{}]", underlying.get(registry).format(registry)),
            TupleType(arr) => {
                if arr.is_empty() {
                    return "()".to_string();
                }

                let mut res = "(".to_string();

                let first = arr[0].get(registry).format(registry);
                res = res + &*first;

                for a in arr[1..].iter() {
                    res = res + ", " + &*a.get(registry).format(registry);
                }

                return res + ")";
            },
            AstType::FunctionsType { name, .. } => format!("{name}(..)"),
            AstType::FunctionType { name, .. } => format!("{name}()"),
            AstType::StructType {name, .. } => name.clone()
        }
    }

}

impl AstType {


    pub fn equals(&self, other: &Self, registry: &TypeRegistry) -> bool {
        self._equals(other, registry, false)
    }

    pub fn loose_equals(&self, other: &Self, registry: &TypeRegistry) -> bool {
        self._equals(other, registry, true)
    }

    pub fn _equals(&self, other: &Self, registry: &TypeRegistry, loose: bool) -> bool {
        match (self, other) {
            // unknown can be derived into any type
            (AstType::UnknownType, _) if loose => true,
            (_, AstType::UnknownType) if loose => true,

            (AstType::Void, AstType::Void) => true,
            (AstType::Bool, AstType::Bool) => true,
            (AstType::Int, AstType::Int) => true,
            (AstType::Float, AstType::Float) => true,
            (AstType::StringType, AstType::StringType) => true,
            (AstType::SymbolType(s1), AstType::SymbolType(s2)) => *s1 == *s2,
            (ReferenceType {underlying: u1}, ReferenceType {underlying: u2}) |
            (NullableType {underlying: u1}, NullableType {underlying: u2}) => {
                u1.get(registry)._equals(&u2.get(registry), registry, loose)
            }
            (ArrayType {underlying: u1}, ArrayType {underlying: u2}) => {
                u1.get(registry)._equals(&u2.get(registry), registry, loose)
            }
            (TupleType(arr1), TupleType(arr2)) => {
                if arr1.len() != arr2.len() {
                    return false;
                }

                for i in 0..arr1.len() {
                    let a = arr1[i];
                    let b = arr2[i];

                    if !a.get(registry)._equals(&b.get(registry), registry, loose) {
                        return false;
                    }
                }

                true
            }

            // TODO comparing names should be fine?
            (StructType {name: n1, ..}, StructType {name: n2, ..}) => *n1 == *n2,

            (NullableType {underlying}, _) if loose => underlying.get(registry)._equals(other, registry, loose),
            (_, NullableType {underlying}) if loose => other._equals(&underlying.get(registry), registry, loose),

             _ => false
        }


    }

}

impl AstType {

    pub fn get_assign_result(&self, other: Self, registry: &mut TypeRegistry) -> Option<Self> {
        if self.equals(&other, registry) {
            return Some(other);
        }

        match (self, other) {
            // let x: float = 1;
            (AstType::Float, AstType::Int) => {
                Some(AstType::Float)
            },
            (NullableType {underlying}, NullableType {underlying: other_underlying}) => {
                if let AstType::UnknownType = underlying.get(registry) {
                    Some(NullableType {underlying: other_underlying})
                } else {
                    if let AstType::UnknownType = other_underlying.get(registry) {
                        Some(self.clone())
                    } else {
                        None
                    }
                }
            },
            (NullableType {underlying}, other) => {
                if let AstType::UnknownType = underlying.get(registry) {
                    let underlying_type = registry.register(other);
                    
                    Some(NullableType {underlying: underlying_type})
                } else {
                    let assign_res = underlying.get(registry).get_assign_result(other, registry);

                    if let Some(res) = assign_res {
                        return Some(NullableType {underlying: registry.register(res)})
                    }

                    None
                }
            }

            _ => None
        }
    }

}

impl AstType {

    // FIXME missing span info for error handling
    pub fn resolve_type(self,registry: &mut TypeRegistry, symbol_table: &mut TypeSymTable) -> Result<AstType, ErrorContext<AnalysisError>> {
        Ok(match self {
            AstType::SymbolType(name) => {
                let opt = symbol_table.get(&name);

                if let Some(t) = opt {
                    return Ok(t.get(registry));
                }

                return Err(ErrorContext::new(ResolutionFailed(name), 0, 0));
            }
            ReferenceType {underlying, .. } => {
                underlying.resolve_type(registry, symbol_table)?;

                ReferenceType {underlying}
            }
            NullableType {underlying} => {
                underlying.resolve_type(registry, symbol_table)?;

                if matches!(underlying.get(registry), NullableType {..}) {
                    return Err(ErrorContext::new(RedundantNullable, 0, 0));
                }

                NullableType {underlying}
            }
            ArrayType {underlying} => {
                underlying.resolve_type(registry, symbol_table)?;

                ArrayType {underlying}
            }
            TupleType(mut arr) => {
                for a in arr.iter_mut() {
                    a.resolve_type(registry, symbol_table)?;
                }

                TupleType(arr)
            }
            FunctionsType { name, mut overloads } => {
                for a in overloads.iter_mut() {
                    a.resolve_type(registry, symbol_table)?;
                }

                FunctionsType {name, overloads}
            }
            FunctionType { name, mut params, return_type } => {
                return_type.resolve_type(registry, symbol_table)?;

                for a in params.iter_mut() {
                    a.resolve_type(registry, symbol_table)?;
                }

                FunctionType {name, params, return_type}
            }
            StructType {name, mut fields, mut children} => {
                for f in fields.iter_mut() {
                    f.0.resolve_type(registry, symbol_table)?;
                }

                for e in children.iter_mut() {
                    let mem = e.1;
                    
                    mem.0.resolve_type(registry, symbol_table)?;
                }

                StructType {name, fields, children}
            }


            _ => self
        })
    }
    
}

impl AstType {
    pub fn get_type_name(&self, registry: &TypeRegistry) -> String {
        match self {
            AstType::Void => "V".to_string(),
            AstType::Bool => "b".to_string(),
            AstType::Int => "i".to_string(),
            AstType::Float => "f".to_string(),
            AstType::StringType => "s".to_string(),
            AstType::SymbolType(n) => format!("L{n};"),
            AstType::ReferenceType { underlying } => underlying.get(registry).get_type_name(registry), // references do not affect method signature
            AstType::ArrayType {underlying, .. } => format!("A{};",underlying.get(registry).get_type_name(registry)),
            AstType::TupleType(types) => {
                let mut name = "T".to_string();
                for t in types {
                    name += t.get(registry).get_type_name(registry).as_str();
                }

                name + ";"
            }
            AstType::FunctionType { .. } => panic!("Functions do not have names!"),
            AstType::StructType {name, .. } => format!("L{name};"),
            AstType::NullableType {underlying} => underlying.get(registry).get_type_name(registry),
            
            _ => panic!("{self:?}")
        }
    }

    pub fn word_size(&self, registry: &TypeRegistry) -> u32 {
        match self {
            AstType::Void => 0,
            AstType::Bool | AstType::Int | AstType::Float | AstType::StringType | AstType::ReferenceType {..} => 1,
            AstType::SymbolType(_) => {
                panic!("Size not known for unresolved symbol {self:?}")  
            },
            AstType::ArrayType { underlying} => 1,
            AstType::TupleType(types) => {
                // just reference
                if true {
                    return 1;
                }

                let mut size = 0;

                for x in types {
                    size += x.get(registry).word_size(registry);
                }
                
                size + 1 // size of types + 1 for reference
            }
            AstType::FunctionType { .. } => 1,
            AstType::StructType {.. } => 1,
            NullableType {underlying} => underlying.get(registry).word_size(registry),
            _ => panic!("Word size not implemented for {self:?}")
        }
    }
    
}
