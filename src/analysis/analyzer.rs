use crate::analysis::symbol_table::{SymbolTable, TypeSymTable};
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::{FunctionType, FunctionsType, StructType};
use crate::ast::ast_type::MemberInfo;
use crate::ast::statement::Statement::{BlockStmt, FunctionStmt, StructStmt};
use crate::ast::statement::{Statement, TypedStmt, UntypedStmt};
use crate::error::analysis_error::AnalysisError::{ExpectedConst, IllegalScopelessStatement, IllegalStatementInStruct, StatementMismatch};
use crate::error::analysis_error::StatementType::Block;
use crate::error::analysis_error::{AnalysisError, EmptyRes};
use crate::error::ok;
use crate::error::runtime_error::ValueTypeVariant;
use std::collections::HashMap;
use crate::analysis::mangling::ManglingVisitor;
use crate::analysis::method_transforms::TransformVisitor;
use crate::analysis::type_resolution::TypeResolver;
use crate::ast::walking::folder::Folder;
use crate::error::context::{ErrorContext, Errors, Span};

pub struct Analyzer {
    root: UntypedStmt,
    symbol_table: TypeSymTable,
    pub errors: Errors<AnalysisError>
}


impl Analyzer {

    pub fn new(root: UntypedStmt) -> Self {
        Self {
            root,
            symbol_table: SymbolTable::new(),
            errors: vec![]
        }
    }

    pub fn analyze(&mut self, registry: &mut TypeRegistry) -> TypedStmt {
        self.record_top_level(registry);

        let mut type_resolver = TypeResolver::new(registry, &mut self.symbol_table);

        Self::record_consts(self.root.clone(), &mut type_resolver, &mut self.errors);

        let mut resolved_root: TypedStmt = self.root.clone().walk_fold(&mut type_resolver);

        if !type_resolver.errors.is_empty() {
            self.errors.append(&mut type_resolver.errors);
            // return Err(type_resolver.errors);
        }

        // TODO semantic analysis would probs be nice xd
        
        let mut mangler = ManglingVisitor::new(registry);
        resolved_root.walk_visit_mut(&mut mangler);

        if !mangler.errors.is_empty() {
            self.errors.append(&mut mangler.errors);
            // return Err(mangler.errors);
        }

        TransformVisitor::transform(registry, &self.symbol_table, &mut resolved_root);

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

                    FunctionStmt {name, params, return_type, .. } => {
                        let t = FunctionType {
                            name: name.clone(),
                            params: params.iter().map(|p| p.param_type.clone()).collect(),
                            return_type: *return_type
                        };

                        let func_type = registry.register(t);

                        self.record_function(registry, func_type);
                    },

                    StructStmt {name, fields, body } => {
                        let mut children = HashMap::new();

                        let mut field_infos = vec![];

                        let mut i = 0;
                        for f in fields {
                            let info = MemberInfo(f.param_type, f.name.clone(), i);
                            children.insert(f.name.clone(), info.clone());

                            field_infos.push(info);

                            i += 1;
                        }
                        let mut table = SymbolTable::new();

                        for x in body {
                            match &x.node {
                                FunctionStmt {name, return_type, params, .. } => {
                                    let t = FunctionType {
                                        name: name.clone(),
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
                            let info = MemberInfo(t.0.clone(), name.clone(), u16::MAX);

                            children.insert(name.clone(), info);
                        }

                        let struct_type = registry.register(StructType {
                            name: name.clone(),
                            fields: field_infos,
                            children
                        });

                        self.symbol_table.record(name.clone(), struct_type);
                    },
                    
                    _ => {
                        self.errors.push(ErrorContext::of(IllegalScopelessStatement(x.clone()), x.span))
                    }
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