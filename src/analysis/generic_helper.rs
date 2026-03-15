use std::collections::HashMap;
use ordermap::OrderMap;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::statement::{Parameter, Statement, StatementBlock, TypedStmt, UntypedStmt};
use crate::ast::statement::Statement::FunctionStmt;
use crate::error::context::Span;
use crate::util::spanned::Spanned;

pub struct GenericHelper {
    generic_functions: HashMap<String, (String, Vec<Parameter>, TypeEntry, StatementBlock<()>, Span)>,
    implemented_functions: HashMap<String, Vec<(OrderMap<String, TypeEntry>, Option<TypedStmt>)>>
}


impl GenericHelper {

    pub fn new() -> Self {
        Self {
            generic_functions: HashMap::new(),
            implemented_functions: HashMap::new()
        }
    }

    pub fn record_generic(&mut self, key: String, name: String, params: Vec<Parameter>, return_type: TypeEntry, body: StatementBlock<()>, span: Span) {
        self.generic_functions.insert(key, (name, params, return_type, body, span));
    }

    pub fn get_generic(&self, registry: &mut TypeRegistry, key: &String) -> Option<UntypedStmt> {
        let (name, params, return_type, body, span) = self.generic_functions.get(key)?.clone();


        let mut cloned_params = vec![];

        for p in params {
            cloned_params.push(Parameter{name: p.name, param_type: p.param_type.duplicate(registry)});
        }

        let cloned_ret_type = return_type.duplicate(registry);

        Some(Spanned::of(
            FunctionStmt {
                name: name.clone(),
                generics: vec![],
                params: cloned_params,
                return_type: cloned_ret_type,
                body
            },
            span
        ))
    }

    pub fn register_implementation(&mut self, key: String, generics: OrderMap<String, TypeEntry>) -> usize {
        if let Some(arr) = self.implemented_functions.get_mut(&key) {
            arr.push((generics, None));

            return arr.len()-1;
        } else {
            self.implemented_functions.insert(key, vec![(generics, None)]);
            return 0;
        }
    }

    pub fn record_implementation(&mut self,key: &String, pos: usize, func: TypedStmt) {
        if let Some(arr) = self.implemented_functions.get_mut(key) {
            arr[pos].1 = Some(func);
        } else {
            panic!("Not registered");
        }
    }

    pub fn has_implementation(&self,registry: &TypeRegistry ,key: &String, generics: OrderMap<String, TypeEntry>) -> bool {
        if let Some(vec) = self.implemented_functions.get(key) {
            for e in vec {
                let gens = &e.0;

                if gens.len() != generics.len() {
                    return false;
                }

                let mut i1 = generics.iter();
                let mut i2 = gens.iter();

                while let Some(g1) = i1.next() && let Some(g2) = i2.next() {
                    if *g1.0 != *g2.0 {
                        return false;
                    }

                    if !g1.1.get(registry).equals(&g2.1.get(registry), registry) {
                        return false;
                    }
                }
            }

            return true;
        }

        false
    }

    pub fn get_implementation(&mut self, key: &String) -> Vec<TypedStmt> {
        if let Some(arr) = self.implemented_functions.get(key) {
            return arr.iter().filter(|e| e.1.is_some()).map(|(_, v) | v.clone().unwrap()).collect();
        }

        vec![]
    }


}