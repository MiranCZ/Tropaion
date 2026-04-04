use std::collections::HashMap;
use ordermap::OrderMap;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::modifier::Modifier;
use crate::ast::statement::{Parameter, Statement, StatementBlock, TypedStmt, UntypedStmt};
use crate::ast::statement::Statement::FunctionStmt;
use crate::error::context::Span;
use crate::util::spanned::Spanned;

pub struct GenericHelper {
    generic_functions: HashMap<String, (String, Modifier, Vec<Parameter>, TypeEntry, StatementBlock<()>, Span)>,
    implemented_functions: HashMap<String, Vec<TypedStmt>>,
    requests: HashMap<String, Vec<(OrderMap<String, TypeEntry>, Option<AstType>)>>
}


impl GenericHelper {

    pub fn new() -> Self {
        Self {
            generic_functions: HashMap::new(),
            implemented_functions: HashMap::new(),
            requests: HashMap::new()
        }
    }

    pub fn record_generic(&mut self, key: String, name: String, modifier: Modifier, params: Vec<Parameter>, return_type: TypeEntry, body: StatementBlock<()>, span: Span) {
        self.generic_functions.insert(key, (name, modifier, params, return_type, body, span));
    }

    pub fn get_generic(&self, registry: &mut TypeRegistry, key: &String) -> Option<UntypedStmt> {
        let (name, modifier, params, return_type, body, span) = self.generic_functions.get(key)?.clone();


        let mut cloned_params = vec![];

        for p in params {
            cloned_params.push(Parameter{name: p.name, param_type: p.param_type.duplicate(registry)});
        }

        let cloned_ret_type = return_type.duplicate(registry);

        Some(Spanned::of(
            FunctionStmt {
                name: name.clone(),
                modifier,
                generics: vec![],
                params: cloned_params,
                return_type: cloned_ret_type,
                body
            },
            span
        ))
    }

    pub fn request_resolution(&mut self, type_registry: &TypeRegistry, name: String, generic_types: OrderMap<String, TypeEntry>, owner: Option<AstType>) {
        if self.has_requested(type_registry, &name, &generic_types) {
            return;
        }

        if let Some(arr) = self.requests.get_mut(&name) {
            arr.push((generic_types, owner));
        } else {
            self.requests.insert(name, vec![(generic_types, owner)]);
        }
    }

    pub fn get_requests(&self) -> &HashMap<String, Vec<(OrderMap<String, TypeEntry>, Option<AstType>)>> {
        &self.requests
    }


    pub fn record_implementation(&mut self,key: String, func: TypedStmt) {
        if let Some(arr) = self.implemented_functions.get_mut(&key) {
            arr.push(func);
        } else {
            self.implemented_functions.insert(key, vec![func]);
        }
    }

    fn has_requested(&self,registry: &TypeRegistry ,key: &String, generics: &OrderMap<String, TypeEntry>) -> bool {
        if let Some(vec) = self.requests.get(key) {

            'loopCheck:
            for e in vec {
                let gens = &e.0;

                if gens.len() != generics.len() {
                    return false;
                }

                let mut i1 = generics.iter();
                let mut i2 = gens.iter();

                while let Some(g1) = i1.next() && let Some(g2) = i2.next() {
                    if *g1.0 != *g2.0 {
                        continue 'loopCheck;
                    }

                    if !g1.1.get(registry).equals(&g2.1.get(registry), registry) {
                        continue 'loopCheck;
                    }
                }

                return true;
            }

            return false;
        }

        false
    }

    pub fn get_implementation(&mut self, key: &String) -> Vec<TypedStmt> {
        if let Some(arr) = self.implemented_functions.get(key) {
            return arr.clone();
        }

        vec![]
    }


}