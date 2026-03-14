use crate::analysis::generic_helper::GenericHelper;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::expression::TypedExpr;
use crate::ast::statement::{Statement, StatementBlock, TypedStmt};
use crate::ast::walking::visitor_mut::VisitorMut;

pub struct GenericFixer<'a> {
    registry: &'a mut TypeRegistry,
    generic_helper: GenericHelper,
}

impl <'a> GenericFixer<'a> {
    pub fn fix(stmt: &mut TypedStmt, registry: &'a mut TypeRegistry, generic_helper: GenericHelper) {
        let mut new = Self {
            registry, generic_helper
        };

        stmt.walk_visit_mut(&mut new);
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
                let mut key = name.clone() + "_";
                for p in params.iter() {
                    key.push_str(p.param_type.get(self.registry).get_type_name(self.registry).as_ref());
                }                
                if matches!(return_type.get(self.registry), AstType::GenericType{..}) {
                    removed.push(key);
                    return false;
                }

                for p in params {
                    // TODO create `is_generic` method for arg
                    if matches!(p.param_type.get(self.registry), AstType::GenericType{..}) {
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

}