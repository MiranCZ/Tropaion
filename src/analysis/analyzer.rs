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
use crate::error::context::{ErrorContext, Span};

pub struct Analyzer {
    root: UntypedStmt,
    symbol_table: TypeSymTable
}


impl Analyzer {

    pub fn new(root: UntypedStmt) -> Self {
        Self {
            root,
            symbol_table: SymbolTable::new()
        }
    }

    pub fn analyze(&mut self, registry: &mut TypeRegistry) -> Result<TypedStmt, ErrorContext<AnalysisError>> {
        self.record_top_level(registry)?;

        let mut type_resolver = TypeResolver::new(registry, &mut self.symbol_table);

        Self::record_consts(self.root.clone(), &mut type_resolver)?;

        let mut resolved_root: TypedStmt = self.root.clone().walk_fold(&mut type_resolver);

        if !type_resolver.errors.is_empty() {
            return Err(type_resolver.errors[0].clone());
        }

        // TODO semantic analysis would probs be nice xd
        
        let mut mangler = ManglingVisitor::new(registry);
        resolved_root.walk_visit_mut(&mut mangler);

        if !mangler.errors.is_empty() {
            return Err(mangler.errors[0].clone());
        }

        TransformVisitor::transform(registry, &self.symbol_table, &mut resolved_root);

        Ok(resolved_root)
    }


    /// record all top-level structs and functions which can be used everywhere
    fn record_top_level(&mut self, registry: &mut TypeRegistry) -> Result<(), ErrorContext<AnalysisError>> {
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

                        self.record_function(registry, func_type)?;
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

                                    Self::_record_function(&mut table,registry ,func_type)?;
                                },

                                Statement::CommentStmt(..) | Statement::MultilineCommentStmt(..) => {}

                                _ => return Err(ErrorContext::of(IllegalStatementInStruct(x.clone()), x.span))
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
                    
                    _ => return Err(ErrorContext::of(IllegalScopelessStatement(x.clone()), x.span))
                }
            }

            return ok();
        }

        Err(ErrorContext::of(StatementMismatch {expected: Block, got: self.root.clone()}, self.root.span))
    }

    fn record_consts(root: UntypedStmt, type_resolver: &mut TypeResolver) -> Result<(), ErrorContext<AnalysisError>> {
        if let BlockStmt{ body } = root.node {
            for x in body {
                match x.node {
                    Statement::VarDeclarationStmt {name, is_const, value, explicit_type} => {
                        if !is_const {
                            return Err(ErrorContext::of(ExpectedConst(name), x.span));
                        }
                        if explicit_type.is_none() {
                            return Err(ErrorContext::of(AnalysisError::TypelessConst, x.span))
                        }

                        type_resolver.fold_var_declaration(name, is_const, value, explicit_type, x.span);
                    },

                    _ => {}
                }
            }

            return ok();
        }

        let span = root.span;
        Err(ErrorContext::of(StatementMismatch {expected: Block, got: root}, span))
    }


    fn record_function(&mut self,registry: &mut TypeRegistry, func: TypeEntry) -> Result<(), ErrorContext<AnalysisError>> {
        Self::_record_function(&mut self.symbol_table,registry ,func)
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