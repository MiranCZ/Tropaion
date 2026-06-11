use std::collections::{HashMap, HashSet};
use ordermap::OrderMap;
use crate::analysis::mangling;
use crate::analysis::symbol_table::{SymbolTable, TypeSymTable, TypeSymTableInfo};
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{FunctionType, FunctionsType, StructType};
use crate::ast::expression::Expression;
use crate::ast::expression::Expression::{CallExpr, IdentifierExpr};
use crate::ast::modifier::Modifier;
use crate::ast::statement::{Parameter, Statement, StatementBlock, TypedStmt};
use crate::ast::statement::Statement::{FunctionStmt, ReturnStmt, VarDeclarationStmt};
use crate::ast::walking::folder::{FoldedExpr, FoldedStmt, Folder};
use crate::error::analysis_error::AnalysisError;
use crate::error::analysis_error::AnalysisError::DanglingConstructor;
use crate::error::context::{ErrorContext, Span};
use crate::util::spanned::Spanned;

pub struct ConstructorLifter<'a> {
    registry: &'a mut TypeRegistry,
    constructors: HashMap<String, TypeEntry>,
    errors: &'a mut Vec<ErrorContext<AnalysisError>>
}


impl<'a> ConstructorLifter<'a> {
    pub fn new(stmt: TypedStmt,errors: &'a mut Vec<ErrorContext<AnalysisError>> ,registry: &'a mut TypeRegistry, symbol_table: &'a mut TypeSymTable) -> TypedStmt{
        let mut constructors = HashMap::new();

        let stmt = ConstructorCollector::collect(stmt, &mut constructors, errors, symbol_table, registry);

        let mut new = Self {
            registry,
            constructors,
            errors
        };


        stmt.walk_fold(&mut new)
    }

}


struct ConstructorCollector<'a> {
    registry: &'a mut TypeRegistry,
    constructors: &'a mut HashMap<String, TypeEntry>,
    symbol_table: &'a mut TypeSymTable,
    owner: Vec<TypeEntry>,
    errors: &'a mut Vec<ErrorContext<AnalysisError>>
}

impl <'a> ConstructorCollector<'a> {

    fn collect(stmt: TypedStmt, constructors: &'a mut HashMap<String, TypeEntry>, errors: &'a mut Vec<ErrorContext<AnalysisError>>, symbol_table: &'a mut TypeSymTable, registry: &'a mut TypeRegistry) -> TypedStmt {
        let mut new = Self{registry, constructors, symbol_table, errors, owner: vec![]};

        stmt.walk_fold(&mut new)
    }

}


impl <'a> Folder<TypeEntry, TypeEntry> for ConstructorCollector<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn fold_annotation(&mut self, t: TypeEntry) -> TypeEntry {
        t
    }

    fn fold_constructor(&mut self, mut modifier: Modifier, params: Vec<Parameter>, mut body: StatementBlock<TypeEntry>, span: Span) -> FoldedStmt<TypeEntry> {
        if let Some(owner) = self.owner.last() {
            let name = "<init>".to_string();

            let owner_name;

            match owner.get(self.registry) {
                StructType {name, ..} |
                AstType::EnumType {name, ..} => {
                    owner_name = name;
                }

                _ => {
                    self.errors.push(ErrorContext::of(AnalysisError::InternalError("No owner for constructor!".to_string()), span));
                    return TypedStmt::err(self.registry, span);
                }
            }

            let mut type_params = vec![];

            for p in params.iter() {
                type_params.push(p.param_type);
            }
            modifier = modifier.with_static();

            let mangled = mangling::mangle_name_type(self.registry, "<init>".to_string(), owner_name.clone(), &type_params);

            let fn_type = self.registry.register(FunctionType {
                name: format!("{owner_name}${name}"),
                modifier,
                generics: OrderMap::new(),
                params: type_params,
                return_type: *owner
            });

            self.constructors.insert(mangled, fn_type);

            self.symbol_table.record(name.clone(), fn_type);

            for b in body.iter_mut() {
                if let Statement::ExpressionStmt(e) = b.node.clone() {
                    if let CallExpr {func, args, ..} = &e.node {

                        b.node = VarDeclarationStmt {
                            name: "this".to_string(),
                            is_const: false,
                            value: e,
                            explicit_type: Some(*owner)
                        }
                    }
                }

            }

            body.push(Spanned::new(ReturnStmt(Some(Spanned::new(IdentifierExpr(*owner, "this".to_string()), 0, 0))), 0, 0));

            FunctionStmt {
                name,
                modifier,
                generics: vec![],
                params,
                body,
                return_type: *owner
            }
        } else {
            self.errors.push(ErrorContext::of(DanglingConstructor, span));

            TypedStmt::err(self.registry, span)
        }
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



    fn fold_call(&mut self, t: TypeEntry, func: Box<Spanned<Expression<TypeEntry>>>, args: Vec<Spanned<Expression<TypeEntry>>>, span: Span) -> FoldedExpr<TypeEntry> {
        if let AstType::StructType {name, fields, ..} = func.get_type().get(self.registry) {
            let mut typed_args = vec![];

            for a in args.iter() {
                typed_args.push(a.get_type());
            }

            let mangled = mangling::mangle_name_type(self.registry, "<init>".to_string(), name.clone(), &typed_args);

            // we are calling an explicit constructor
            if let Some(constructor) = self.constructors.get(&mangled) {
                let mangled = format!("{name}$<init>");

                return CallExpr {
                    t: func.get_type(),
                    func: Box::new(Spanned::of(IdentifierExpr(*constructor, mangled), func.span)),
                    args
                };
            } else {
                if args.len() != fields.len() {
                    self.errors.push(ErrorContext::of(AnalysisError::illegal_func_args(name, args.clone(), self.registry), span));
                } else {
                    let mut i1 = fields.iter();
                    let mut i2 = args.iter();

                    while let Some(v1) = i1.next() && let Some(v2) = i2.next() {
                        if !v1.typ.get(self.registry).loose_equals(&v2.get_type().get(self.registry), self.registry) {
                            self.errors.push(ErrorContext::of(AnalysisError::illegal_func_args(name, args.clone(), self.registry), span));
                            break;
                        }
                    }
                }
            }

        }

        CallExpr { t, func, args }
    }


}