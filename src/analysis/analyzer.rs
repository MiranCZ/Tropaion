use crate::analysis::symbol_table::{SymbolTable, TypeSymTable};
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::{FunctionType, FunctionsType, GenericType, StructType, UnknownType};
use crate::ast::ast_type::MemberInfo;
use crate::ast::statement::Statement::{BlockStmt, FunctionStmt, StructStmt};
use crate::ast::statement::{Statement, TypedStmt, UntypedStmt};
use crate::error::analysis_error::AnalysisError::{ExpectedConst, IllegalScopelessStatement, IllegalStatementInStruct, StatementMismatch};
use crate::error::analysis_error::StatementType::Block;
use crate::error::analysis_error::{AnalysisError, EmptyRes};
use crate::error::ok;
use crate::error::runtime_error::ValueTypeVariant;
use std::collections::HashMap;
use ordermap::OrderMap;
use crate::analysis::constant_folding::ConstExprFolder;
use crate::analysis::generic_fixer::GenericFixer;
use crate::analysis::mangling::ManglingVisitor;
use crate::analysis::method_transforms::TransformVisitor;
use crate::analysis::top_level_collector::TopLevelCollector;
use crate::analysis::type_resolution::TypeResolver;
use crate::analysis::unique_name_checker::UniqueNameChecker;
use crate::ast::walking::folder::Folder;
use crate::error::context::{ErrorContext, Errors, Span};
use crate::intrinsics::type_injector::{get_injected_functions, get_injected_structs};

pub struct Analyzer {
    root: UntypedStmt,
    symbol_table: TypeSymTable,
    type_table: TypeSymTable,
    pub errors: Errors<AnalysisError>
}


impl Analyzer {

    pub fn new(root: UntypedStmt) -> Self {
        Self {
            root,
            symbol_table: SymbolTable::new(),
            type_table: SymbolTable::new(),
            errors: vec![]
        }
    }

    pub fn analyze(&mut self, registry: &mut TypeRegistry) -> TypedStmt {
        let mut type_resolver = TypeResolver::new(registry, &mut self.symbol_table, &mut self.type_table);

        TopLevelCollector::collect(&mut type_resolver, self.root.clone());

        Self::record_consts(self.root.clone(), &mut type_resolver, &mut self.errors);

        let mut resolved_root: TypedStmt = self.root.clone().walk_fold(&mut type_resolver);
        type_resolver.resolve_generic_funcs();

        let mut errors = type_resolver.errors;
        let generic_helper = type_resolver.generic_helper;

        GenericFixer::fix(&mut resolved_root, registry, generic_helper);

        if !errors.is_empty() {
            self.errors.append(&mut errors);
        }

        // TODO semantic analysis would probs be nice xd
        
        self.errors.append(&mut UniqueNameChecker::check(registry, &resolved_root));
        
        let mut mangler = ManglingVisitor::new(registry);
        resolved_root.walk_visit_mut(&mut mangler);

        if !mangler.errors.is_empty() {
            self.errors.append(&mut mangler.errors);
            // return Err(mangler.errors);
        }

        TransformVisitor::transform(registry, &self.symbol_table, &mut resolved_root);
        let resolved_root = ConstExprFolder::fold(resolved_root, registry);

        resolved_root
    }


    /// record all top-level structs and functions which can be used everywhere
    fn record_top_level(&mut self, registry: &mut TypeRegistry) {
        if let BlockStmt{ body } = &self.root.node.clone() {
            for x in body {
                match &x.node {
                    Statement::CommentStmt(_) | Statement::MultilineCommentStmt(_) => {},
                    Statement::VarDeclarationStmt {..} => {
                        // will resolve after functions and structs
                    },

                    FunctionStmt {name, modifier, generics, params, return_type, .. } => {
                        let mut resolved_generics = OrderMap::new();

                        self.type_table.push();
                        for g in generics {
                            resolved_generics.insert(g.clone(), registry.register(UnknownType));
                            self.type_table.record(g.clone(), registry.register(GenericType {name: g.clone()}));
                        }

                        let t = FunctionType {
                            name: name.clone(),
                            modifier: *modifier,
                            generics: resolved_generics,
                            params: params.iter().map(|p| p.param_type.clone()).collect(),
                            return_type: *return_type
                        };

                        self.type_table.pop();
                        let func_type = registry.register(t);

                        self.record_function(registry, func_type);
                    },

                    StructStmt {name, fields, body, generics } => {
                        let mut children = HashMap::new();

                        let mut field_infos = vec![];

                        let mut i = 0;
                        for f in fields {
                            let info = MemberInfo::new(f.param_type, f.name.clone(), i);
                            children.insert(f.name.clone(), info.clone());

                            field_infos.push(info);

                            i += 1;
                        }
                        let mut table = SymbolTable::new();

                        for x in body {
                            match &x.node {
                                FunctionStmt {name, modifier,generics, return_type, params, .. } => {
                                    let mut resolved_generics = OrderMap::new();

                                    for g in generics {
                                        resolved_generics.insert(g.clone(), registry.register(UnknownType));
                                    }

                                    let t = FunctionType {
                                        name: name.clone(),
                                        modifier: *modifier,
                                        generics: resolved_generics,
                                        return_type: *return_type,
                                        params: params.iter().map(|p| p.clone().param_type).collect()
                                    };
                                    let func_type = registry.register(t);

                                    let res = Self::_record_function(&mut table,registry ,func_type);

                                    match res {
                                        Ok(_) => {}
                                        Err(e) => self.errors.push(e)
                                    }
                                },

                                Statement::CommentStmt(..) | Statement::MultilineCommentStmt(..) => {}

                                _ => {
                                    self.errors.push(ErrorContext::of(IllegalStatementInStruct(x.clone()), x.span))
                                }
                            }
                        }

                        for e in table.last().unwrap().iter() {
                            let t = e.1;
                            let name = e.0;

                            // functions don't have order
                            let info = MemberInfo::new(t.0.clone(), name.clone(), u16::MAX);

                            children.insert(name.clone(), info);
                        }

                        let mut resolved_generics = OrderMap::new();

                        for g in generics {
                            resolved_generics.insert(g.clone(), registry.register(UnknownType));
                        }

                        let struct_type = registry.register(StructType {
                            name: name.clone(),
                            fields: field_infos,
                            children,
                            generics: resolved_generics
                        });

                        self.symbol_table.record(name.clone(), struct_type);
                        self.type_table.record(name.clone(), struct_type);
                    },
                    
                    _ => {
                        self.errors.push(ErrorContext::of(IllegalScopelessStatement(x.clone()), x.span))
                    }
                }
            }
            
            for func in get_injected_functions(registry) {
                let t = registry.register(func);
                self.record_function(registry, t);
            }
            for struct_type in get_injected_structs(registry) {
                if let StructType {name, ..} = struct_type.clone() {
                    let struct_type = registry.register(struct_type);

                    self.symbol_table.record(name.clone(), struct_type);
                    self.type_table.record(name.clone(), struct_type);
                } else {
                    panic!("Invalid injected {struct_type:?}");
                }
            }

            return;
        }

        self.errors.push(ErrorContext::of(StatementMismatch {expected: Block, got: self.root.clone()}, self.root.span))
    }

    fn record_consts(root: UntypedStmt, type_resolver: &mut TypeResolver, errors: &mut Errors<AnalysisError>) {
        if let BlockStmt{ body } = root.node {
            for x in body {
                match x.node {
                    Statement::VarDeclarationStmt {name, is_const, value, explicit_type} => {
                        if !is_const {
                            errors.push(ErrorContext::of(ExpectedConst(name.clone()), x.span));
                        }
                        if explicit_type.is_none() {
                            errors.push(ErrorContext::of(AnalysisError::TypelessConst, x.span));
                        }

                        type_resolver.fold_var_declaration(name, is_const, value, explicit_type, x.span);
                    },

                    _ => {}
                }
            }

            return;
        }

        let span = root.span;

        errors.push(ErrorContext::of(StatementMismatch {expected: Block, got: root}, span))
    }


    fn record_function(&mut self,registry: &mut TypeRegistry, func: TypeEntry) {
        let res = Self::_record_function(&mut self.symbol_table,registry ,func);

        match res {
            Ok(_) => {}
            Err(e) => self.errors.push(e)
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
                    panic!("Invalid overload {name} {:?}", t.get(registry).format(registry))
                }
            }
        } else {
            // FIXME add span
            return Err(ErrorContext::of(AnalysisError::type_mismatch(ValueTypeVariant::Function, func, registry), Span::new(0, 0)));
        }
        
        ok()
    }

}