use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::{ArrayType, EnumType, ErroredType, NullableType, ReferenceType, StructType, SymbolType, TupleType};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use ordermap::OrderMap;
use crate::ast::modifier::Modifier;

#[derive(Debug, PartialEq, Clone)]
pub enum AstType {
    ErroredType,
    
    UnknownType,
    Void,
    Bool,
    Int,
    Float,
    StringType,
    SymbolType {
        name: String,
        generics: Vec<TypeEntry>
    },
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
        modifier: Modifier,
        generics: OrderMap<String, TypeEntry>,
        params: Vec<TypeEntry>,
        return_type: TypeEntry
    },
    StructType {
        name: String,
        generics: OrderMap<String, TypeEntry>,
        fields: Vec<MemberInfo>, 
        // fields and methods
        children: HashMap<String, MemberInfo>,
    },
    EnumType {
        name: String,
        values: Vec<String>
    },
    GenericType {
        name: String
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct MemberInfo{
    pub typ: TypeEntry,
    pub name: String,
    pub index: u16
}

impl MemberInfo {
    pub fn new(typ: TypeEntry, name: String, index: u16) -> MemberInfo {
        MemberInfo{typ, name, index }
    }

}

impl AstType {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl TypeEntry {

    pub fn err(registry: &mut TypeRegistry) -> TypeEntry {
        registry.register(ErroredType)
    }

    pub fn is_err(&self, registry: &TypeRegistry) -> bool {
        matches!(self.get(registry), ErroredType)
    }

}


impl AstType {

    pub fn format(&self, registry: &TypeRegistry) -> String {
        match self {
            AstType::ErroredType => "$err$".to_string(),
            AstType::UnknownType => "$unknown$".to_string(),
            AstType::Void => "void".to_string(),
            AstType::Bool => "bool".to_string(),
            AstType::Int => "int".to_string(),
            AstType::Float => "float".to_string(),
            AstType::StringType => "string".to_string(),
            AstType::SymbolType{name, ..} => format!("$symbol-{name}$"), // TODO display generics

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
            AstType::FunctionType { name, params,.. } => {
                let mut result = format!("{name}(");

                if !params.is_empty() {
                    let mut iter = params.iter();

                    result.push_str(iter.next().unwrap().format(registry).as_str());

                    for arg in iter {
                        result.push_str(arg.format(registry).as_str());
                        result.push_str(", ");
                    }
                }

                result.push_str(")");
                
                result
            },
            AstType::StructType {name,generics, .. } => {
                let mut res = name.clone();

                if !generics.is_empty() {
                    res.push('<');

                    let mut iter = generics.values();
                    res.push_str(iter.next().unwrap().get(registry).format(registry).as_ref());

                    for i in iter {
                        res.push_str(", ");

                        res.push_str(i.get(registry).format(registry).as_ref());
                    }

                    res.push('>');

                }

                res
            }
            AstType::EnumType {name, ..} => name.clone(),
            AstType::GenericType {name, ..} => format!("{name}")
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

            (AstType::GenericType {..}, _) if loose => true,
            (_, AstType::GenericType {..}) if loose => true,

            (AstType::UnknownType, AstType::UnknownType) => true,
            (AstType::GenericType {name: n1}, AstType::GenericType {name: n2}) => n1 == n2,

            (AstType::Void, AstType::Void) => true,
            (AstType::Bool, AstType::Bool) => true,
            (AstType::Int, AstType::Int) => true,
            (AstType::Float, AstType::Float) => true,
            (AstType::StringType, AstType::StringType) => true,
            (AstType::SymbolType{name: n1, generics: g1}, AstType::SymbolType{name: n2, generics: g2}) => {
                if n1 != n2 {
                    return false;
                }

                Self::type_arrays_equal(&registry, loose, g1, g2)
            },
            (ReferenceType {underlying: u1}, ReferenceType {underlying: u2}) |
            (NullableType {underlying: u1}, NullableType {underlying: u2}) => {
                u1.get(registry)._equals(&u2.get(registry), registry, loose)
            }
            (ArrayType {underlying: u1}, ArrayType {underlying: u2}) => {
                u1.get(registry)._equals(&u2.get(registry), registry, loose)
            }
            (TupleType(arr1), TupleType(arr2)) => {
                Self::type_arrays_equal(&registry, loose, arr1, arr2)
            }

            // TODO comparing names should be fine?
            (StructType {name: n1,generics: g1, ..}, StructType {name: n2, generics: g2, ..}) => {
                if *n1 != *n2 {
                    return false;
                }

                if g1.len() != g2.len() {
                    return false;
                }

                let mut it1 = g1.iter();
                let mut it2 = g2.iter();

                while let Some(v1) = it1.next() && let Some(v2) = it2.next() {
                    if v1.0 != v2.0 {
                        return false;
                    }

                    let a = v1.1;
                    let b = v2.1;

                    if !a.get(registry)._equals(&b.get(registry), registry, loose) {
                        return false;
                    }
                }

                true
            }
            (EnumType {name: n1, values: v1}, EnumType {name: n2, values: v2}) => {
                if *n1 != *n2 {
                    return false;
                }

                if v1.len() != v2.len() {
                    return false;
                }

                let mut it1 = v1.iter();
                let mut it2 = v2.iter();

                while let Some(x1) = it1.next() && let Some(x2) = it2.next() {
                    if *x1 != *x2 {
                        return false
                    }
                }

                true
            }

            (NullableType {underlying}, _) if loose => underlying.get(registry)._equals(other, registry, loose),
            (_, NullableType {underlying}) if loose => other._equals(&underlying.get(registry), registry, loose),

             _ => false
        }


    }

    fn type_arrays_equal(registry: &&TypeRegistry, loose: bool, arr1: &Vec<TypeEntry>, arr2: &Vec<TypeEntry>) -> bool {
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
}

impl AstType {

    pub fn get_assign_result(&self, other: Self, registry: &mut TypeRegistry) -> Option<Self> {
        if self.equals(&other, registry) {
            return Some(other);
        }

        match (self, other) {
            (AstType::UnknownType, other) => {
                Some(other)
            }
            (other, AstType::UnknownType) => {
                Some(other.clone())
            }


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
            },

            (StructType {name: n1, generics: g1, fields, children }, StructType {name: n2, generics: g2, .. }) => {
                if *n1 != *n2 {
                    return None;
                }

                if g1.len() != g2.len() {
                    return None;
                }

                let mut assigned_generics = OrderMap::new();

                let mut it1 = g1.iter();
                let mut it2 = g2.iter();

                while let Some(v1) = it1.next() && let Some(v2) = it2.next() {
                    if v1.0 != v2.0 {
                        return None;
                    }

                    let a = v1.1.get(registry);
                    let b = v2.1.get(registry);

                    if let Some(res) = a.get_assign_result(b, registry) {
                        assigned_generics.insert(v1.0.clone(), registry.register(res));
                    } else {
                        return None;
                    }
                }

                return Some(StructType {
                    name: n1.clone(),
                    generics: assigned_generics,
                    fields: fields.clone(),
                    children: children.clone()
                })
            }
            _ => None
        }
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
            AstType::SymbolType{name: n, ..} => format!("L{n};"),
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
            AstType::StructType {name,generics, .. } => {
                let mut name = format!("L{name}_");

                for t in generics {
                    name += t.1.get(registry).get_type_name(registry).as_str();
                }

                name + ";"
            },
            AstType::NullableType {underlying} => underlying.get(registry).get_type_name(registry),
            AstType::GenericType {name} => "g".to_string(),
            AstType::UnknownType => "?".to_string(),
            _ => panic!("{self:?}")
        }
    }

    pub fn word_size(&self, registry: &TypeRegistry) -> u32 {
        if true {
            if matches!(self, AstType::Void) {
                return 0;
            }
            return 1;
        }
        
        match self {
            AstType::Void => 0,
            AstType::Bool | AstType::Int | AstType::Float | AstType::StringType | AstType::ReferenceType {..} => 1,
            AstType::SymbolType{..} => {
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
