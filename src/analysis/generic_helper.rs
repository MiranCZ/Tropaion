use std::collections::HashMap;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::statement::{Parameter, Statement, StatementBlock, TypedStmt, UntypedStmt};
use crate::ast::statement::Statement::FunctionStmt;
use crate::error::context::Span;
use crate::util::spanned::Spanned;

pub struct GenericHelper {
    generic_functions: HashMap<String, (String, Vec<Parameter>, TypeEntry, StatementBlock<()>, Span)>,
    implemented_functions: HashMap<String, Vec<TypedStmt>>
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

    pub fn get_generic(&self, registry: &mut TypeRegistry, name: &String) -> Option<UntypedStmt> {


        let (name, params, return_type, body, span) = self.generic_functions.get(name)?.clone();


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

    pub fn record_implementation(&mut self, key: String, func: TypedStmt) {
        if let Some(arr) = self.implemented_functions.get_mut(&key) {
            arr.push(func);
        } else {
            self.implemented_functions.insert(key, vec![func]);
        }
    }

    pub fn collect_implemented(&mut self) -> Vec<TypedStmt> {
        let mut result = vec![];

        for arr in self.implemented_functions.values_mut() {
            result.append(arr);
        }

        result
    }


}