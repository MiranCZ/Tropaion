use std::collections::HashMap;
use crate::analysis::symbol_table::{SymbolTable, TypeSymTable};
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::UnknownType;
use crate::error::analysis_error::{AnalysisError, EmptyRes};
use crate::error::context::ErrorContext;
use crate::error::ok;

type TypeEntryKey = u64;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TypeEntry {
    key: TypeEntryKey
}

impl TypeEntry {
   
    pub fn new_instance(&self, parent: &mut TypeRegistry) -> Self {
        let t = self.get(parent);
       
        parent.register(t)
    }
    
    pub fn get(&self, parent: &TypeRegistry) -> AstType {
        parent.get(self.key)
    }
    
    pub fn duplicate(&self, parent: &mut TypeRegistry) -> TypeEntry {
        let resolved = self.get(parent);
        
        parent.register(resolved)
    }
    
    pub fn mutate(&self, parent: &mut TypeRegistry,new_value: AstType) {
        parent.registry.insert(self.key, new_value);
    }
    
}

#[derive(Debug)]
pub struct TypeRegistry {
    registry: HashMap<TypeEntryKey, AstType>,
    i: u64
}


impl TypeRegistry {

    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            i: 0
        }
    }

    pub fn register(&mut self, typ: AstType) -> TypeEntry {
        let ind = self.i;
        self.registry.insert(ind, typ);

        self.i += 1;

        TypeEntry{key: ind}
    }

    pub fn get(&self, key: TypeEntryKey) -> AstType {
        let stored = self.registry.get(&key);

        if let Some(v) = stored {
            return v.clone();
        }

        UnknownType
    }

}