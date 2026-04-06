use std::collections::{HashMap, HashSet};
use ordermap::OrderMap;
use crate::analysis::symbol_table::{SymbolTable, TypeSymTable, TypeSymTableInfo};
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{FunctionType, FunctionsType};
use crate::ast::expression::Expression;
use crate::ast::expression::Expression::{CallExpr, IdentifierExpr};
use crate::ast::modifier::Modifier;
use crate::ast::statement::{Parameter, Statement, StatementBlock};
use crate::ast::statement::Statement::FunctionStmt;
use crate::ast::walking::folder::{FoldedExpr, FoldedStmt, Folder};
use crate::error::analysis_error::AnalysisError;
use crate::error::context::{ErrorContext, Span};
use crate::util::spanned::Spanned;

pub struct ConstructorLifter<'a> {
    registry: &'a mut TypeRegistry,
    owner: Vec<TypeEntry>,
    symbol_table: &'a mut TypeSymTable,
    constructors: HashMap<String, TypeEntry>,
    pub errors: Vec<ErrorContext<AnalysisError>>
}


impl<'a> ConstructorLifter<'a> {
    pub fn new(registry: &'a mut TypeRegistry, symbol_table: &'a mut TypeSymTable) -> Self {
        Self {
            registry,
            symbol_table,
            constructors: HashMap::new(),
            owner: vec![],
            errors: vec![]
        }
    }

}


impl <'a> Folder<TypeEntry, TypeEntry> for ConstructorLifter<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn fold_annotation(&mut self, t: TypeEntry) -> TypeEntry {
        t
    }

    fn fold_constructor(&mut self, modifier: Modifier, params: Vec<Parameter>, body: StatementBlock<TypeEntry>) -> FoldedStmt<TypeEntry> {
        if let Some(owner) = self.owner.last() {
            let name = "<init>".to_string();

            let mut type_params = vec![];

            for p in params.iter() {
                type_params.push(p.param_type);
            }

            let fn_type = self.registry.register(FunctionType {
                name: name.clone(),
                modifier,
                generics: OrderMap::new(),
                params: type_params,
                return_type: *owner
            });

            self.constructors.insert(name.clone(), fn_type);

            println!("RECORDED {name}");
            self.symbol_table.record(name.clone(), fn_type);

            FunctionStmt {
                name,
                modifier,
                generics: vec![],
                params,
                body,
                return_type: *owner
            }
        } else {
            panic!("No owner for constructor?")
        }
    }

    fn fold_call(&mut self, t: TypeEntry, func: Box<Spanned<Expression<TypeEntry>>>, args: Vec<Spanned<Expression<TypeEntry>>>, span: Span) -> FoldedExpr<TypeEntry> {
        println!("CALLING {func:?} {:?}", func.get_type().get(self.registry));

        if let IdentifierExpr(t, ..) = func.node {
            println!("\t{:?}",t.get(self.registry));
        }
        if let AstType::StructType {name, ..} = func.get_type().get(self.registry) {
            let mangled = "<init>".to_string();

            println!("CALLING CONSTRUCTOR {}", mangled);

            let constructor = self.constructors.get(&mangled).unwrap();

            let mangled = format!("{name}$<init>");

            return CallExpr {
                t: *constructor,
                func: Box::new(Spanned::of(IdentifierExpr(*constructor, mangled), func.span)),
                args
            };
        } else {
        }

        CallExpr { t, func, args }
    }

    fn fold_struct(&mut self, name: String, public_constructor: bool, fields: Vec<Parameter>, body: StatementBlock<TypeEntry>, generics: Vec<String>, span: Span) -> FoldedStmt<TypeEntry> {
        self.owner.push(self.symbol_table.get(&name).unwrap());

        let folded_body = self.fold_block(body);

        self.owner.pop();

        let folded_fields = fields
            .into_iter()
            .map(|p| Parameter {
                name: p.name,
                param_type: self.fold_type_entry(p.param_type),
            })
            .collect();


        Statement::StructStmt {
            name,
            public_constructor,
            fields: folded_fields,
            body: folded_body,
            generics
        }
    }
}