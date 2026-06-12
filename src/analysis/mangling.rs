use std::collections::{HashMap, HashSet};
use ordermap::OrderMap;
use crate::analysis::symbol_table::SymbolTable;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::{FunctionType, FunctionsType, NullableType, StructType};
use crate::ast::ast_type::{AstType, MemberInfo};
use crate::ast::expression::Expression::{IdentifierExpr, MemberExpr};
use crate::ast::expression::TypedExpr;
use crate::ast::modifier::Modifier;
use crate::ast::statement::{Parameter, StatementBlock};
use crate::ast::statement::Statement::FunctionStmt;
use crate::ast::walking::visitor_mut::VisitorMut;
use crate::error::analysis_error::AnalysisError;
use crate::error::context::{ErrorContext, Span};

pub struct ManglingVisitor<'a> {
    registry: &'a mut TypeRegistry,
    owner_table: SymbolTable<String, String>,
    // Maps method name → concrete return TypeEntry, populated by visit_mut_call.
    // Used in visit_mut_identifier so that methods whose generic T only appears in
    // the return type still get unique mangled names per specialization.
    concrete_return_table: SymbolTable<TypeEntry, ()>,
    resolved: HashSet<TypeEntry>,
    pub errors: Vec<ErrorContext<AnalysisError>>
}


impl<'a> ManglingVisitor<'a> {
    pub fn new(registry: &'a mut TypeRegistry) -> Self {
        Self {
            registry,
            resolved: HashSet::new(),
            owner_table: SymbolTable::new(),
            concrete_return_table: SymbolTable::<TypeEntry, ()>::new(),
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
        // Constructors encode the owner in their call-site name already; exclude return type
        // to keep definitions and call sites consistent.
        let ret = if owner.is_empty() || name == "<init>" { None } else { Some(*return_type) };

        *name = mangle_name(self.registry, name.clone(), owner.clone(), params, ret);

        for s in body {
            s.walk_visit_mut(self);
        }

        for p in params {
            self.visit_mut_type(&mut p.param_type);
        }
        self.visit_mut_type(return_type);
    }

    fn visit_mut_struct(&mut self, name: &mut String, pc: &mut bool, fields: &mut Vec<Parameter>, body: &mut StatementBlock<TypeEntry>, generics: &mut Vec<String>, span: Span) {
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

    fn visit_mut_call(&mut self, t: &mut TypeEntry, func: &mut TypedExpr, args: &mut [TypedExpr], span: Span) {
        // Extract the callee name so we can store the concrete return type for
        // visit_mut_identifier.  The call expression's type `t` is the concrete
        // return type resolved by the type system, while the identifier's function
        // type may still carry a generic T in the return position.
        let fn_name = match &func.node {
            IdentifierExpr(_, name) => Some(name.clone()),
            MemberExpr { property, .. } => {
                if let IdentifierExpr(_, name) = &property.node {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None
        };

        if let Some(name) = fn_name {
            self.concrete_return_table.push();
            self.concrete_return_table.record(name, *t);

            self.visit_mut_type(t);
            self.visit_mut_expr(func);
            for a in args { self.visit_mut_expr(a); }

            self.concrete_return_table.pop();
        } else {
            self.visit_mut_type(t);
            self.visit_mut_expr(func);
            for a in args { self.visit_mut_expr(a); }
        }
    }

    fn visit_mut_identifier(&mut self, t: &mut TypeEntry, name: &mut String, span: Span) {
        let owner = self.owner_table.get_or(name, String::new());

        self.visit_mut_type(t);

        if let FunctionType{params, return_type, ..} = t.get(self.registry) {
            let ret = if owner.is_empty() {
                None
            } else if matches!(return_type.get(self.registry), AstType::UnknownType) {
                // UnknownType return (e.g. address.__load_at) has one bytecode impl for all types;
                // don't include the return type in the mangled name.
                None
            } else {
                self.concrete_return_table.get(name)
            };
            *name = mangle_name_type(self.registry, name.clone(), owner.clone(), &params, ret);
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

        // Do not include return type here: function types in the registry may still
        // carry generic T in the return position, which would produce wrong names
        // like "Vec$get_ig".  Identifier names are set correctly by visit_mut_identifier
        // using concrete_return_table, and function definition names are set by
        // visit_mut_function using the concrete return type from the AST.
        *name = mangle_name_type(self.registry, name.clone(), owner.clone(), &params, None);

        for g in generics.values_mut() {
            self.visit_mut_type(g);
        }

        self.visit_mut_type(return_type);
    }

    fn visit_mut_struct_type(&mut self, name: &mut String, generics: &mut OrderMap<String, TypeEntry>, constructors: &mut Vec<TypeEntry>, fields: &mut Vec<MemberInfo>, children: &mut HashMap<String, MemberInfo>) {
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




pub fn mangle_name(registry: &TypeRegistry, name: String, owner: String, params: &Vec<Parameter>, return_type: Option<TypeEntry>) -> String {
    let mut name = from_owner(name, owner) + "_";

    for p in params {
        if p.param_type.is_err(registry) {
            return format!("{name}_<err>");
        }

        name += p.param_type.get(registry).get_type_name(registry).as_str();
    }

    if let Some(ret) = return_type {
        name += ret.get(registry).get_type_name(registry).as_str();
    }

    name
}

pub fn mangle_name_type(registry: &TypeRegistry, name: String, owner: String, params: &Vec<TypeEntry>, return_type: Option<TypeEntry>) -> String {
    let mut name = from_owner(name, owner) + "_";

    for p in params {
        if p.is_err(registry) {
            return format!("{name}_<err>");
        }

        name += p.get(registry).get_type_name(registry).as_str();
    }

    if let Some(ret) = return_type {
        name += ret.get(registry).get_type_name(registry).as_str();
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
