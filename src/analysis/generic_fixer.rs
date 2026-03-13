use crate::analysis::generic_helper::GenericHelper;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
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

    fn visit_mut_block(&mut self, body: &mut StatementBlock<TypeEntry>) {
        body.retain_mut(|b| {
            return if let Statement::FunctionStmt { generics, params, return_type, .. } = &mut b.node {
                if matches!(return_type.get(self.registry), AstType::GenericType{..}) {
                    return false;
                }
                
                for p in params {
                    // TODO create `is_generic` method for arg
                    if matches!(p.param_type.get(self.registry), AstType::GenericType{..}) {
                        return false;
                    }
                }

                generics.is_empty()
            } else {
                true
            }
        });

        for b in body.iter_mut() {
            b.walk_visit_mut(self);
        }

        for func in self.generic_helper.collect_implemented() {
            body.push(func);
        }
    }

}