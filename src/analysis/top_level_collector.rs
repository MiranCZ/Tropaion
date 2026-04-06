use std::collections::HashMap;
use ordermap::OrderMap;
use Statement::EnumStmt;
use crate::analysis::generic_fixer::GenericChecker;
use crate::analysis::mangling;
use crate::analysis::symbol_table::{SymbolTable, TypeSymTable};
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::analysis::type_resolution::TypeResolver;
use crate::ast::ast_type::AstType::{ConstructorType, EnumType, FunctionType, FunctionsType, GenericType, StructType, UnknownType};
use crate::ast::ast_type::{AstType, MemberInfo};
use crate::ast::expression::Expression;
use crate::ast::modifier::Modifier;
use crate::ast::statement::{Parameter, Statement, StatementBlock, UntypedStmt};
use crate::ast::statement::Statement::{BlockStmt, FunctionStmt, StructStmt};
use crate::ast::walking::folder::{FoldedExpr, FoldedStmt, Folder};
use crate::ast::walking::visitor::Visitor;
use crate::error::analysis_error::AnalysisError;
use crate::error::analysis_error::AnalysisError::IllegalStatementInStruct;
use crate::error::context::{ErrorContext, Span};
use crate::error::ok;
use crate::error::runtime_error::ValueTypeVariant;
use crate::intrinsics::type_injector::{get_injected_functions, get_injected_structs};
use crate::util::spanned::Spanned;

pub struct TopLevelCollector<'a, 'b> {
    resolver: &'a mut TypeResolver<'b>
}

impl <'a, 'b> TopLevelCollector<'a, 'b> {
    pub fn new(resolver: &'a mut TypeResolver<'b>) -> Self {
        Self {
            resolver
        }
    }

    fn construct_enum_type(name: &String, values: &Vec<String>) -> AstType {
        EnumType {
            name: name.clone(),
            values: values.clone()
        }
    }

    pub fn collect(resolver: &'a mut TypeResolver<'b>, stmt: UntypedStmt) {
        let mut new = Self::new(resolver);

        if let BlockStmt {body} = &stmt.node {
            for s in body {
                match &s.node {
                    StructStmt {name, ..} => {
                        let t = new.resolver.registry.register(UnknownType);
                        new.resolver.type_table.record(name.clone(), t);
                    },
                    EnumStmt {name, values, ..} => {
                        let t = new.resolver.registry.register(Self::construct_enum_type(name, values));
                        new.resolver.type_table.record(name.clone(), t);
                        new.resolver.symbol_table.record(name.clone(), t);
                    }
                    _ => {}
                }
            }
        }


        for func in get_injected_functions(new.resolver.registry) {
            let t = new.resolver.registry.register(func);
            new.record_function(t);
        }
        for struct_type in get_injected_structs(new.resolver.registry) {
            if let StructType {name, ..} = struct_type.clone() {
                let struct_type = new.resolver.registry.register(struct_type);

                new.resolver.symbol_table.record(name.clone(), struct_type);
                new.resolver.type_table.record(name.clone(), struct_type);
            } else {
                panic!("Invalid injected {struct_type:?}");
            }
        }

        stmt.walk_fold(&mut new);
    }


    fn record_function(&mut self, func: TypeEntry) {
        let res = Self::_record_function(&mut self.resolver.symbol_table,self.resolver.registry ,func);

        match res {
            Ok(_) => {}
            // Err(e) => self.errors.push(e)
            Err(e) => {}
        }
    }

    fn _record_function(symbol_table: &mut TypeSymTable, registry: &mut TypeRegistry, func: TypeEntry) -> Result<(), ErrorContext<AnalysisError>> {
        if let FunctionType {name, ..} = func.get(registry) {
            let t = symbol_table.get(&name);

            if t.is_none() {
                let mut overloads = vec![];
                overloads.push(func);
                symbol_table.record(name.clone(), registry.register(FunctionsType {
                    name, overloads
                }));
            } else if let Some(t) = t {
                if let FunctionsType { mut overloads, ..} = t.get(registry) {
                    overloads.push(func);
                    t.mutate(registry, FunctionsType {name, overloads});
                } else {
                    // FIXME add span
                    return Err(ErrorContext::of(AnalysisError::NameAlreadyUsed(name), Span::new(0,0)));
                }
            }
        } else {
            // FIXME add span
            return Err(ErrorContext::of(AnalysisError::type_mismatch(ValueTypeVariant::Function, func, registry), Span::new(0, 0)));
        }

        ok()
    }
}

// FIXME should be visitor, but visitor does not exist for untyped
impl <'a, 'b> Folder<(), ()> for TopLevelCollector<'a, 'b> {
    fn get_registry(&self) -> &TypeRegistry {
        self.resolver.get_registry()
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.resolver.get_registry_mut()
    }

    fn fold_expr(&mut self, expr: Spanned<Expression<()>>) -> Spanned<FoldedExpr<()>> {
        expr
    }

    fn fold_function(&mut self, name: String, modifier: Modifier, generics: Vec<String>, params: Vec<Parameter>, return_type: TypeEntry, body: StatementBlock<()>, span: Span) -> FoldedStmt<()> {
        let func_type = self.resolve_func_signature(&name, &modifier, &generics, &params, &return_type, &body, &span, String::new());

        self.record_function(func_type);

        Statement::FunctionStmt {
            name,
            modifier,
            generics,
            params,
            return_type,
            body,
        }
    }

    fn fold_struct(&mut self, name: String, pc: bool, fields: Vec<Parameter>, body: StatementBlock<()>, generics: Vec<String>, span: Span) -> FoldedStmt<()> {
        // let t = self.resolver.registry.register(UnknownType);
        let t = self.resolver.type_table.get(&name).unwrap();
        self.resolve_struct_signature(t, &name, &pc, &fields, &body, &generics, &span);

        self.resolver.symbol_table.record(name.clone(), t);
        self.resolver.type_table.record(name.clone(), t);

        StructStmt {
            name, fields, body, generics,
            public_constructor: pc
        }
    }



    fn fold_annotation(&mut self, _: ()) -> () {
    }
}

impl <'a, 'b> TopLevelCollector<'a, 'b> {
    fn resolve_func_signature(&mut self, name: &String, modifier: &Modifier, generics: &Vec<String>, params: &Vec<Parameter>, return_type: &TypeEntry, body: &StatementBlock<()>, span: &Span, owner: String) -> TypeEntry {
        let mut resolved_generics = OrderMap::new();

        self.resolver.type_table.push();
        for g in generics.iter() {
            resolved_generics.insert(g.clone(), self.resolver.registry.register(UnknownType));
            self.resolver.type_table.record(g.clone(), self.resolver.registry.register(GenericType {name: g.clone()}));
        }

        let mut resolved_params = vec![];

        let mut has_generics_params = false;
        for p in params.iter() {
            let dup = p.param_type.duplicate(self.resolver.registry);
            let param_t = self.resolver.fold_type_entry(dup);
            resolved_params.push(param_t);

            if GenericChecker::is_generic(param_t, self.resolver.registry) {
                has_generics_params = true;
            }
        }

        let resolved_return = self.resolver.fold_type_entry(*return_type);

        if !generics.is_empty() || has_generics_params ||  GenericChecker::is_generic(*return_type, self.resolver.registry) {
            let key = mangling::from_owner(name.clone(), owner.clone());

            self.resolver.generic_helper.record_generic(key, name.clone(), *modifier, params.clone(), *return_type, body.clone(), *span);
        }

        let t = FunctionType {
            name: name.clone(),
            modifier: *modifier,
            generics: resolved_generics,
            params: resolved_params,
            return_type: resolved_return
        };

        self.resolver.type_table.pop();
        let func_type = self.resolver.registry.register(t);

        func_type
    }

    fn get_func_key(&self, name: String, owner: String, params: &Vec<Parameter>) -> String {
        mangling::mangle_name(self.resolver.registry, name, owner, &params)
    }

    fn get_func_key_type(&self, name: String, owner: String, params: &Vec<TypeEntry>) -> String {
        mangling::mangle_name_type(self.resolver.registry, name, owner, &params)
    }

    fn resolve_struct_signature(&mut self, type_entry: TypeEntry ,name: &String, public_constructor: &bool, fields: &Vec<Parameter>, body: &StatementBlock<()>, generics: &Vec<String>, span: &Span) {
        let mut struct_type = self.get_registry_mut().register(UnknownType);

        let mut children = HashMap::new();

        let mut field_infos = vec![];

        self.resolver.type_table.push();
        for g in generics {
            self.resolver.type_table.record(g.clone(), self.resolver.registry.register(GenericType {name: g.clone()}));
        }

        let mut i = 0;
        for f in fields {
            let resolved_param = self.resolver.fold_type_entry(f.param_type);

            let info = MemberInfo::new(resolved_param, f.name.clone(), i);
            children.insert(f.name.clone(), info.clone());

            field_infos.push(info);

            i += 1;
        }


        let mut constructors = vec![];
        let mut table = SymbolTable::new();

        for s in body {
            match &s.node {
                Statement::FunctionStmt { name: fn_name, modifier, generics, params, return_type, body,  } => {
                    let func_type = self.resolve_func_signature(fn_name, modifier, generics, params, return_type, body, &s.span, name.clone());

                    let res = Self::_record_function(&mut table,self.resolver.registry ,func_type);


                    match res {
                        Ok(_) => {}
                        Err(e) => panic!("{e:?}")
                    }
                }
                Statement::ConstructorStmt {modifier, params, body} => {
                    let mut param_types = vec![];
                    for x in params {
                        param_types.push(x.param_type);
                    }

                    constructors.push(self.resolver.registry.register(
                        ConstructorType {
                            modifier: *modifier,
                            params: param_types,
                            owner: struct_type
                        }
                    ));
                },

                _ => {}
            }
        }

        // default constructor
        {
            let mut param_types = vec![];

            for f in fields {
                param_types.push(f.param_type);
            }

            let mut modifier = Modifier::new();

            if *public_constructor {
                modifier = modifier.public().unwrap();
            }

            constructors.push(self.resolver.registry.register(
                ConstructorType {
                    modifier,
                    params: param_types,
                    owner: struct_type
                }
            ));
        }

        for e in table.last().unwrap().iter() {
            let t = e.1;
            let name = e.0;

            // functions don't have order
            let info = MemberInfo::new(t.0.clone(), name.clone(), u16::MAX);

            children.insert(name.clone(), info);
        }

        self.resolver.type_table.pop();

        let mut resolved_generics = OrderMap::new();

        for g in generics {
            resolved_generics.insert(g.clone(), self.resolver.registry.register(UnknownType));
        }

        struct_type.mutate(self.resolver.registry, StructType {
            name: name.clone(),
            constructors,
            fields: field_infos,
            children,
            generics: resolved_generics
        });

        type_entry.mutate(self.resolver.registry, struct_type.get(self.resolver.registry));

        // struct_type

    }

}


