use std::collections::HashMap;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::UnknownType;
use crate::ast::expression::TypedExpr;
use crate::ast::statement::{TypedStmt, UntypedStmt};
use crate::ast::walking::folder::Folder;

pub struct TypeDuplicator<'a> {
    registry: &'a mut TypeRegistry,
    duplicate_cache: HashMap<TypeEntry, TypeEntry>
}

impl <'a> TypeDuplicator<'a> {
    pub fn new(registry: &'a mut TypeRegistry) -> Self {
        Self {
            registry,
            duplicate_cache: HashMap::new()
        }
    }

    pub fn duplicate(registry: &'a mut TypeRegistry,expr: TypedExpr) -> TypedExpr {
        expr.walk_fold(&mut Self::new(registry))
    }

}

impl <'a> Folder<TypeEntry, TypeEntry> for TypeDuplicator<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn fold_annotation(&mut self, t: TypeEntry) -> TypeEntry {
        self.fold_type_entry(t)
    }

    fn fold_type_entry(&mut self, t: TypeEntry) -> TypeEntry {
        if let Some(cache) = self.duplicate_cache.get(&t) {
            return *cache;
        }
        let result = self.registry.register(UnknownType);

        // cache before deepening
        self.duplicate_cache.insert(t, result);

        let resolved = t.get(self.registry);

        let duplicated = resolved.walk_fold(self);

        result.mutate(self.registry, duplicated);


        result
    }

}

// FIXME copypasted struct
pub struct UntypedTypeDuplicator<'a> {
    registry: &'a mut TypeRegistry,
    duplicate_cache: HashMap<TypeEntry, TypeEntry>
}

impl <'a> UntypedTypeDuplicator<'a> {
    pub fn new(registry: &'a mut TypeRegistry) -> Self {
        Self {
            registry,
            duplicate_cache: HashMap::new()
        }
    }

    pub fn duplicate(registry: &'a mut TypeRegistry,stmt: UntypedStmt) -> UntypedStmt {
        stmt.walk_fold(&mut Self::new(registry))
    }

}

impl <'a> Folder<(), ()> for UntypedTypeDuplicator<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn fold_annotation(&mut self, _: ()) -> () {
    }

    fn fold_type_entry(&mut self, t: TypeEntry) -> TypeEntry {
        if let Some(cache) = self.duplicate_cache.get(&t) {
            return *cache;
        }
        let result = self.registry.register(UnknownType);

        // cache before deepening
        self.duplicate_cache.insert(t, result);

        let resolved = t.get(self.registry);

        let duplicated = resolved.walk_fold(self);

        result.mutate(self.registry, duplicated);


        result
    }

}

