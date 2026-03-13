use std::collections::HashMap;
use crate::analysis::symbol_table::TypeSymTable;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::expression::TypedExpr;
use crate::ast::statement::{Parameter, StatementBlock, TypedStmt};
use crate::ast::walking::visitor_mut::VisitorMut;
use crate::error::context::Span;

pub struct TransformVisitor<'a> {
    registry: &'a mut TypeRegistry,
    symbol_table: &'a TypeSymTable,
    inside_struct: Option<TypeEntry>,
}


impl<'a> TransformVisitor<'a> {
    pub fn new(registry: &'a mut TypeRegistry, symbol_table: &'a TypeSymTable) -> Self {
        Self {
            registry, symbol_table,
            inside_struct: None
        }
    }

    pub fn transform(registry: &'a mut TypeRegistry, symbol_table: &'a TypeSymTable, stmt: &mut TypedStmt) {
        let mut new = Self::new(registry, symbol_table);
        stmt.walk_visit_mut(&mut new);
    }

    fn with_inside<F>(&mut self, opt: Option<TypeEntry>, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let old = std::mem::replace(&mut self.inside_struct, opt);
        f(self);
        self.inside_struct = old;
    }
}

impl <'a> VisitorMut<'a> for TransformVisitor<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn visit_mut_function(&mut self, name: &mut String, generics: &mut Vec<String>, params: &mut Vec<Parameter>, return_type: &mut TypeEntry, body: &mut StatementBlock<TypeEntry>, span: Span) {
        if let Some(t) = self.inside_struct {
            params.insert(0, Parameter{name: "this".to_string(), param_type: t});
        }
    }

    fn visit_mut_struct(&mut self, name: &mut String, fields: &mut Vec<Parameter>, body: &mut StatementBlock<TypeEntry>, generics: &mut Vec<String>, span: Span) {
        self.with_inside(Some(self.symbol_table.get(&name).unwrap()), |visitor| {
            for s in body {
                s.walk_visit_mut(visitor);
            }
        });
    }

    fn visit_mut_expr(&mut self, expr: &mut TypedExpr) {
        // don't care about expressions
    }

    fn visit_mut_type(&mut self, typ: &mut TypeEntry) {
        // don't care about types
    }

}
