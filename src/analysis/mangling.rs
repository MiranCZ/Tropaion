use std::collections::HashMap;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::{FunctionType, FunctionsType, NullableType, StructType};
use crate::ast::ast_type::{AstType, MemberInfo};
use crate::ast::expression::Expression::IdentifierExpr;
use crate::ast::expression::TypedExpr;
use crate::ast::statement::{Parameter, StatementBlock};
use crate::ast::walking::visitor_mut::VisitorMut;
use crate::error::analysis_error::AnalysisError;
use crate::error::context::{ErrorContext, Span};

pub struct ManglingVisitor<'a> {
    registry: &'a mut TypeRegistry,
    owner: String,
    pub errors: Vec<ErrorContext<AnalysisError>>
}


impl<'a> ManglingVisitor<'a> {
    pub fn new(registry: &'a mut TypeRegistry) -> Self {
        Self {
            registry,
            owner: String::new(),
            errors: vec![]
        }
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

impl <'a> VisitorMut<'a> for ManglingVisitor<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn visit_mut_function(&mut self, name: &mut String, generics: &mut Vec<String>, params: &mut Vec<Parameter>, return_type: &mut TypeEntry, body: &mut StatementBlock<TypeEntry>, span: Span) {
        *name = from_owner(name.clone(), self.owner.clone());
        *name = mangle_name(self.registry, name.clone(), params);

        for s in body {
            s.walk_visit_mut(self);
        }

        for p in params {
            p.param_type.walk_visit_mut(self);
        }
        return_type.walk_visit_mut(self);
    }

    fn visit_mut_struct(&mut self, name: &mut String, fields: &mut Vec<Parameter>, body: &mut StatementBlock<TypeEntry>, generics: &mut Vec<String>, span: Span) {
        let struct_owner = if self.owner.is_empty() {
            name.clone()
        } else {
            self.owner.clone() // FIXME: same as before
        };

        self.with_owner(struct_owner, |visitor| {
            for f in fields {
                f.param_type.walk_visit_mut(visitor);
            }

            for s in body {s.walk_visit_mut(visitor); }
        });
    }

    fn visit_mut_identifier(&mut self, t: &mut TypeEntry, name: &mut String, span: Span) {
        t.walk_visit_mut(self);

        if let FunctionType{params, ..} = t.get(self.registry) {
            *name = from_owner(name.clone(), self.owner.clone());
            *name = mangle_name_type(self.registry, name.clone(), &params);
        }
    }

    fn visit_mut_member(&mut self, t: &mut TypeEntry, member: &mut TypedExpr, property: &mut TypedExpr, null_safe: &mut bool, span: Span) {
        t.walk_visit_mut(self);

        member.walk_visit_mut(self);


        let mut repl = self.owner.clone();

        let mut typ = member.get_type().get(self.registry);
        if let NullableType {underlying} = typ {
            if !*null_safe {
                self.errors.push(ErrorContext::of(AnalysisError::NullableAccess, span));
            }
            typ = underlying.get(self.registry);
        }

        if let StructType {name, ..} = typ {
            repl = if self.owner.is_empty() {
                name.clone()
            } else {
                // owner + "_" + name.as_str()
                // FIXME nested owners shouldn't be possible?
                self.owner.clone()
            };
        }
        let owner = repl;

        self.with_owner(owner, |visitor| {
            property.walk_visit_mut(visitor);
        });
    }

    fn visit_mut_function_type(&mut self, name: &mut String, generics: &mut HashMap<String, TypeEntry>, params: &mut Vec<TypeEntry>, return_type: &mut TypeEntry) {
        *name = from_owner(name.clone(), self.owner.clone());
        *name = mangle_name_type(self.registry, name.clone(), &params);

        for g in generics.values_mut() {
            self.visit_mut_type(g);
        }

        return_type.walk_visit_mut(self);
    }

    fn visit_mut_struct_type(&mut self, name: &mut String, generics: &mut HashMap<String, TypeEntry>, fields: &mut Vec<MemberInfo>, children: &mut HashMap<String, MemberInfo>) {
        let owner = if self.owner.is_empty() {
            name.clone()
        } else {
            // owner + "_" + name.as_str()
            // FIXME nested owners shouldn't be possible?
            self.owner.clone()
        };

        for g in generics.values_mut() {
            self.visit_mut_type(g);
        }

        self.with_owner(owner, |visitor| {
            for i in children.values_mut() {
                if matches!(i.0.get(visitor.registry), FunctionType {..}) || matches!(i.0.get(visitor.registry), FunctionsType {..}) {
                    i.0.walk_visit_mut(visitor);
                }
            }
        });
    }

}



fn from_owner(name: String, owner: String) -> String {
    if owner.is_empty() {
        name
    } else {
        owner + "$" + name.as_str()
    }
}

fn mangle_name(registry: &TypeRegistry, name: String, params: &Vec<Parameter>) -> String {
    let mut name = name + "_";

    for p in params {
        if p.param_type.is_err(registry) {
            return format!("{name}_<err>");
        }

        name += p.param_type.get(registry).get_type_name(registry).as_str();
    }

    name
}

fn mangle_name_type(registry: &TypeRegistry, name: String, params: &Vec<TypeEntry>) -> String {
    let mut name = name + "_";

    for p in params {
        name += p.get(registry).get_type_name(registry).as_str();
    }

    name
}