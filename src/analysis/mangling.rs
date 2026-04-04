use std::collections::{HashMap, HashSet};
use ordermap::OrderMap;
use crate::analysis::symbol_table::SymbolTable;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::{FunctionType, FunctionsType, NullableType, StructType};
use crate::ast::ast_type::{AstType, MemberInfo};
use crate::ast::expression::Expression::IdentifierExpr;
use crate::ast::expression::TypedExpr;
use crate::ast::modifier::Modifier;
use crate::ast::statement::{Parameter, StatementBlock};
use crate::ast::statement::Statement::FunctionStmt;
use crate::ast::walking::visitor_mut::VisitorMut;
use crate::error::analysis_error::AnalysisError;
use crate::error::context::{ErrorContext, Span};

pub struct ManglingVisitor<'a> {
    registry: &'a mut TypeRegistry,
    // owner: String,
    owner_table: SymbolTable<String, String>,
    resolved: HashSet<TypeEntry>,
    pub errors: Vec<ErrorContext<AnalysisError>>
}


impl<'a> ManglingVisitor<'a> {
    pub fn new(registry: &'a mut TypeRegistry) -> Self {
        Self {
            registry,
            resolved: HashSet::new(),
            owner_table: SymbolTable::new(),
            errors: vec![]
        }
    }

}

impl <'a> VisitorMut<'a> for ManglingVisitor<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn visit_mut_type(&mut self, typ: &mut TypeEntry) {
        if !self.resolved.contains(typ) {
            self.resolved.insert(*typ);
            typ.walk_visit_mut(self);
        }
    }

    fn visit_mut_function(&mut self, name: &mut String, modifier: &mut Modifier, generics: &mut Vec<String>, params: &mut Vec<Parameter>, return_type: &mut TypeEntry, body: &mut StatementBlock<TypeEntry>, span: Span) {
        let owner = self.owner_table.get_or(name, String::new());

        *name = mangle_name(self.registry, name.clone(), owner.clone(), params);

        for s in body {
            s.walk_visit_mut(self);
        }

        for p in params {
            self.visit_mut_type(&mut p.param_type);
        }
        self.visit_mut_type(return_type);
    }

    fn visit_mut_struct(&mut self, name: &mut String, fields: &mut Vec<Parameter>, body: &mut StatementBlock<TypeEntry>, generics: &mut Vec<String>, span: Span) {
        self.owner_table.push();
        for b in body.iter() {
            if let FunctionStmt {name: fn_name, ..} = &b.node {
                self.owner_table.record(fn_name.clone(), name.clone());
            }
        }

        for f in fields {
            self.visit_mut_type(&mut f.param_type);
        }

        for s in body {s.walk_visit_mut(self); }

        self.owner_table.pop();
    }

    fn visit_mut_identifier(&mut self, t: &mut TypeEntry, name: &mut String, span: Span) {
        let owner = self.owner_table.get_or(name, String::new());

        self.visit_mut_type(t);

        if let FunctionType{params, ..} = t.get(self.registry) {
            *name = mangle_name_type(self.registry, name.clone(), owner.clone(), &params);
        }
    }

    fn visit_mut_member(&mut self, t: &mut TypeEntry, member: &mut TypedExpr, property: &mut TypedExpr, null_safe: &mut bool, span: Span) {
        self.visit_mut_type(t);

        member.walk_visit_mut(self);


        let mut typ = member.get_type().get(self.registry);
        if let NullableType {underlying} = typ {
            if !*null_safe {
                self.errors.push(ErrorContext::of(AnalysisError::NullableAccess, span));
            }
            typ = underlying.get(self.registry);
        }

        let mut struct_scope = false;
        if let StructType {name, children, ..} = typ {
            struct_scope = true;
            self.owner_table.push();
            for ch in children {
                self.owner_table.record(ch.0, name.clone());
            }
        }

        property.walk_visit_mut(self);

        if struct_scope {
            self.owner_table.pop();
        }
    }

    fn visit_mut_function_type(&mut self, name: &mut String, modifier: &mut Modifier, generics: &mut OrderMap<String, TypeEntry>, params: &mut Vec<TypeEntry>, return_type: &mut TypeEntry) {
        let owner = self.owner_table.get_or(name, String::new());

        *name = mangle_name_type(self.registry, name.clone(), owner.clone(), &params);

        for g in generics.values_mut() {
            self.visit_mut_type(g);
        }

        self.visit_mut_type(return_type);
    }

    fn visit_mut_struct_type(&mut self, name: &mut String, generics: &mut OrderMap<String, TypeEntry>, fields: &mut Vec<MemberInfo>, children: &mut HashMap<String, MemberInfo>) {
        for g in generics.values_mut() {
            self.visit_mut_type(g);
        }

        self.owner_table.push();
        for i in children.values_mut() {
            match i.typ.get(self.registry) {
                FunctionType {name: fn_name, ..} |
                FunctionsType {name: fn_name, ..} => {
                    self.owner_table.record(name.clone(), fn_name);
                }
                _ => {}
            }
        }

        for i in children.values_mut() {
            self.visit_mut_type(&mut i.typ);
        }

        self.owner_table.pop();
    }

}





pub fn mangle_name(registry: &TypeRegistry, name: String, owner: String, params: &Vec<Parameter>) -> String {
    let mut name = from_owner(name, owner) + "_";

    for p in params {
        if p.param_type.is_err(registry) {
            return format!("{name}_<err>");
        }

        name += p.param_type.get(registry).get_type_name(registry).as_str();
    }

    name
}

pub fn mangle_name_type(registry: &TypeRegistry, name: String, owner: String, params: &Vec<TypeEntry>) -> String {
    let mut name = from_owner(name, owner) + "_";

    for p in params {
        name += p.get(registry).get_type_name(registry).as_str();
    }

    name
}

pub fn from_owner(name: String, owner: String) -> String {
    if owner.is_empty() {
        name
    } else {
        owner + "$" + name.as_str()
    }
}