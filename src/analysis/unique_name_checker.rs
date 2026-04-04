use std::collections::HashSet;
use crate::analysis::mangling;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::expression::TypedExpr;
use crate::ast::modifier::Modifier;
use crate::ast::statement::{Parameter, StatementBlock, TypedStmt};
use crate::ast::walking::visitor::Visitor;
use crate::error::analysis_error::AnalysisError;
use crate::error::context::{ErrorContext, Span};

pub struct UniqueNameChecker<'a> {
    registry: &'a mut TypeRegistry,
    errors: Vec<ErrorContext<AnalysisError>>,
    class_like: HashSet<String>,
    functions: HashSet<String>,
    owners: Vec<String>
}

impl <'a> UniqueNameChecker<'a> {
    pub fn check(registry: &'a mut TypeRegistry, stmt: &TypedStmt) -> Vec<ErrorContext<AnalysisError>> {
        let mut errors = vec![];
        let mut class_like = HashSet::new();

        ClassLikeCollector::collect(registry, &mut errors, &mut class_like, stmt);

        let mut owners = vec![];
        owners.push(String::new());
        let mut new = UniqueNameChecker{registry, errors, class_like, functions: HashSet::new(), owners};
        stmt.walk_visit(&mut new);

        new.errors
    }
}

impl <'a> Visitor<'a> for UniqueNameChecker<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn visit_struct(&mut self, name: &String, fields: &Vec<Parameter>, body: &StatementBlock<TypeEntry>, generics: &Vec<String>, span: Span) {
        self.owners.push(name.clone());

        self.visit_block(body);

        self.owners.pop();
    }

    fn visit_enum(&mut self, name: &String, values: &Vec<String>, body: &StatementBlock<TypeEntry>, span: Span) {
        self.owners.push(name.clone());

        self.visit_block(body);

        self.owners.pop();
    }

    fn visit_function(&mut self, name: &String, modifier: &Modifier, generics: &Vec<String>, params: &Vec<Parameter>, return_type: &TypeEntry, body: &StatementBlock<TypeEntry>, span: Span) {
        if self.class_like.contains(name) {
            self.errors.push(ErrorContext::of(AnalysisError::NameAlreadyUsed(name.clone()), span));
            return;
        }

        let key = mangling::mangle_name(self.registry, name.clone(), self.owners.last().unwrap().clone(), params);

        if self.functions.contains(&key) {
            self.errors.push(ErrorContext::of(AnalysisError::function_already_defined(name.clone(), params, self.registry), span));
        }

        self.functions.insert(key);
    }

}


struct ClassLikeCollector<'a> {
    registry: &'a mut TypeRegistry,
    errors: &'a mut Vec<ErrorContext<AnalysisError>>,
    class_like: &'a mut HashSet<String>
}

impl <'a> ClassLikeCollector<'a> {

    fn collect(registry: &'a mut TypeRegistry, errors: &'a mut Vec<ErrorContext<AnalysisError>>, class_like: &'a mut HashSet<String>, stmt: &TypedStmt) {
        let mut new = Self{registry, errors, class_like};
        stmt.walk_visit(&mut new);
    }

    fn update(&mut self, name: &String, span: Span) {
        if self.class_like.contains(name) {
            self.errors.push(ErrorContext::of(AnalysisError::NameAlreadyUsed(name.clone()), span));
            return;
        }

        self.class_like.insert(name.clone());
    }
}

impl <'a> Visitor<'a> for ClassLikeCollector<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn visit_expr(&mut self, expr: &TypedExpr) {
    }

    fn visit_type(&mut self, typ: &TypeEntry) {
    }

    fn visit_struct(&mut self, name: &String, fields: &Vec<Parameter>, body: &StatementBlock<TypeEntry>, generics: &Vec<String>, span: Span) {
        // FIXME this is the span of the whole struct, not of the name
        self.update(name, span);
    }

    fn visit_enum(&mut self, name: &String, values: &Vec<String>, body: &StatementBlock<TypeEntry>, span: Span) {
        // FIXME this is the span of the whole struct, not of the name
        self.update(name, span);
    }



}