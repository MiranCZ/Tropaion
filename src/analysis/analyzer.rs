use crate::analysis::symbol_table::{SymbolTable, TypeSymTable};
use crate::ast::ast_type::AstType::{FunctionType, FunctionsType, StructType};
use crate::ast::ast_type::{AstType, MemberInfo};
use crate::ast::statement::Statement::{BlockStmt, FunctionStmt, StructStmt};
use crate::ast::statement::{Statement, TypedStmt, UntypedStmt};
use std::collections::HashMap;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};

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

    pub fn analyze(&mut self, registry: &mut TypeRegistry) -> TypedStmt {
        self.record_top_level(registry);
        self.record_consts(registry);


        let resolved_root: TypedStmt = self.root.clone().resolve_type(registry, &mut self.symbol_table);

        // TODO semantic analysis would probs be nice xd

        let resolved_root = resolved_root.mangle_functions(registry).transform_methods(registry, &self.symbol_table);

        resolved_root
    }


    /// record all top-level structs and functions which can be used everywhere
    fn record_top_level(&mut self, registry: &mut TypeRegistry) {
        if let BlockStmt{ body } = &self.root.clone() {
            for x in body {
                match x {
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
                            match x {
                                FunctionStmt {name, return_type, params, .. } => {
                                    let t = FunctionType {
                                        name: name.clone(),
                                        return_type: *return_type,
                                        params: params.iter().map(|p| p.clone().param_type).collect()
                                    };
                                    let func_type = registry.register(t);

                                    Self::_record_function(&mut table,registry ,func_type);
                                },
                                _ => panic!("invalid statement inside struct {x:?}")
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
                    _ => panic!("Invalid statement {x:?}")
                }
            }

            return;
        }

        panic!("not a block statement? {:?}",self.root)
    }

    fn record_consts(&mut self, registry: &mut TypeRegistry) {
        if let BlockStmt{ body } = self.root.clone() {
            for x in body {
                match x {
                    Statement::VarDeclarationStmt {name, is_const, value, explicit_type} => {
                        if !is_const {
                            panic!("Top-level variables must be constant!")
                        }

                        let inferred_type = value.resolve_type(registry, &mut self.symbol_table);

                        if let Some(explicit) = explicit_type.clone() {
                            if explicit != inferred_type.get_type() {
                                panic!("Explicit is not the same as inferred {:?} {:?}", explicit_type, inferred_type);
                            }

                            // TODO assignable from
                        }

                        self.symbol_table.record(name, inferred_type.get_type());
                    },

                    _ => {}
                }
            }

            return;
        }

        panic!("not a block statement? {:?}",self.root)
    }


    fn record_function(&mut self,registry: &mut TypeRegistry, func: TypeEntry) {
        Self::_record_function(&mut self.symbol_table,registry ,func)
    }

    fn _record_function(symbol_table: &mut TypeSymTable, registry: &mut TypeRegistry, func: TypeEntry) {
        if let FunctionType {name, ..} = func.get(registry) {
            let t = symbol_table.get(name.clone());

            if t.is_none() {
                let mut overloads = vec![];
                overloads.push(func);
                symbol_table.record(name.clone(), func);
            } else if let Some(t) = t {
                if let FunctionsType { mut overloads, ..} = t.get(registry) {
                    overloads.push(func);
                } else {
                    panic!("Invalid overload {name}")
                }
            }
        } else {
            panic!("{func:?}")
        }
    }

}