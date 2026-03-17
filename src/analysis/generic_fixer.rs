use std::collections::HashSet;
use crate::analysis::generic_helper::GenericHelper;
use crate::analysis::mangling;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::expression::TypedExpr;
use crate::ast::statement::{Parameter, Statement, StatementBlock, TypedStmt};
use crate::ast::walking::visitor::Visitor;
use crate::ast::walking::visitor_mut::VisitorMut;
use crate::error::context::Span;

pub struct GenericFixer<'a> {
    registry: &'a mut TypeRegistry,
    generic_helper: GenericHelper,
    owner: String,
}

impl <'a> GenericFixer<'a> {
    pub fn fix(stmt: &mut TypedStmt, registry: &'a mut TypeRegistry, generic_helper: GenericHelper) {
        let mut new = Self {
            registry, generic_helper,
            owner: String::new()
        };

        stmt.walk_visit_mut(&mut new);
    }

    fn with_owner<F>(&mut self, new_owner: String, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let old = std::mem::replace(&mut self.owner, new_owner);
        f(self);
        self.owner = old;
    }

}

impl <'a> VisitorMut<'a> for GenericFixer<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn visit_mut_type(&mut self, typ: &mut TypeEntry) {
    }

    fn visit_mut_expr(&mut self, expr: &mut TypedExpr) {
    }

    fn visit_mut_block(&mut self, body: &mut StatementBlock<TypeEntry>) {
        let mut removed = vec![];

        body.retain_mut(|b| {
            return if let Statement::FunctionStmt { name, generics, params, return_type, .. } = &mut b.node {
                let key = mangling::mangle_name(self.registry, name.clone(), self.owner.clone(), params);

                if GenericChecker::is_generic(*return_type, self.registry) {
                    removed.push(key);
                    return false;
                }

                for p in params {
                    if GenericChecker::is_generic(p.param_type, self.registry) {
                        removed.push(key);
                        return false;
                    }
                }

                if !generics.is_empty() {
                    removed.push(key);
                    return false;
                }

                return true;
            } else {
                true
            }
        });

        for r in removed {
            for func in self.generic_helper.get_implementation(&r) {
                body.push(func);
            }
        }

        for b in body.iter_mut() {
            b.walk_visit_mut(self);
        }
    }

    fn visit_mut_struct(&mut self, name: &mut String, fields: &mut Vec<Parameter>, body: &mut StatementBlock<TypeEntry>, generics: &mut Vec<String>, span: Span) {
        self.with_owner(name.clone(), |ctx| {
            ctx.visit_mut_block(body);
        });
    }

}

pub struct GenericChecker<'a> {
    registry: &'a mut TypeRegistry,
    visited: HashSet<TypeEntry>,
    is_generic: bool
}

impl <'a> GenericChecker<'a> {
    pub fn is_generic(entry: TypeEntry, registry: &'a mut TypeRegistry) -> bool {
        let mut new = Self {registry, is_generic: false, visited: HashSet::new()};
        entry.walk_visit(&mut new);

        new.is_generic
    }
}

impl <'a> Visitor<'a> for GenericChecker<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn visit_type(&mut self, typ: &TypeEntry) {
        if self.visited.contains(typ) {
            return;
        }
        self.visited.insert(*typ);


        typ.walk_visit(self);
    }

    fn visit_generic_type(&mut self, name: &String) {
        self.is_generic = true;
    }
}