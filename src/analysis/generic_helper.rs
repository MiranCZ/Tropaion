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
        let already = self.has_requested(type_registry, &name, &generic_types, &owner);
        if already {
            return;
        }

        self.add_request(name.clone(), generic_types.clone(), owner.clone());

        // When any method of a struct specialization is first requested, eagerly request
        // all sibling generic methods of the same struct. This ensures that internal
        // cross-calls between methods (e.g. pop(int) calling pop()) are always resolved.
        if let Some(AstType::StructType { name: struct_name, .. }) = &owner {
            let prefix = format!("{}$", struct_name);
            let siblings: Vec<String> = self.generic_functions.keys()
                .filter(|k| k.starts_with(&prefix) && **k != name)
                .cloned()
                .collect();
            for sibling in siblings {
                if !self.has_requested(type_registry, &sibling, &generic_types, &owner) {
                    self.add_request(sibling, generic_types.clone(), owner.clone());
                }
            }
        }
    }

    fn add_request(&mut self, name: String, generic_types: OrderMap<String, TypeEntry>, owner: Option<AstType>) {
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

    fn has_requested(&self, registry: &TypeRegistry, key: &String, generics: &OrderMap<String, TypeEntry>, owner: &Option<AstType>) -> bool {
        if let Some(vec) = self.requests.get(key) {

            'loopCheck:
            for e in vec {
                let gens = &e.0;
                let req_owner = &e.1;

                if gens.len() != generics.len() {
                    continue 'loopCheck;
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

                match (req_owner, owner) {
                    (None, None) => {}
                    (Some(AstType::StructType { generics: g1, .. }), Some(AstType::StructType { generics: g2, .. })) => {
                        if g1.len() != g2.len() {
                            continue 'loopCheck;
                        }
                        let mut i1 = g1.iter();
                        let mut i2 = g2.iter();
                        while let Some(v1) = i1.next() && let Some(v2) = i2.next() {
                            if v1.0 != v2.0 {
                                continue 'loopCheck;
                            }
                            if !v1.1.get(registry).equals(&v2.1.get(registry), registry) {
                                continue 'loopCheck;
                            }
                        }
                    }
                    _ => continue 'loopCheck
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