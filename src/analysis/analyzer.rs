use crate::analysis::symbol_table::{SymbolTable, TypeSymTable};
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::{ConstructorType, FunctionType, FunctionsType, GenericType, StructType, UnknownType};
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
use crate::analysis::contructor_lifter::ConstructorLifter;
use crate::analysis::generic_fixer::GenericFixer;
use crate::analysis::mangling::ManglingVisitor;
use crate::analysis::method_transforms::TransformVisitor;
use crate::analysis::this_validator::ThisValidator;
use crate::analysis::top_level_collector::TopLevelCollector;
use crate::analysis::type_resolution::TypeResolver;
use crate::analysis::unique_name_checker::UniqueNameChecker;
use crate::ast::modifier::Modifier;
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

        self.errors.append(&mut ThisValidator::collect_errors(&resolved_root, registry));

        let mut lifter = ConstructorLifter::new(registry, &mut self.symbol_table);
        resolved_root = resolved_root.walk_fold(&mut lifter);
        self.errors.append(&mut lifter.errors);

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