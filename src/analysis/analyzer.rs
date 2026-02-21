use std::collections::HashMap;
use crate::analysis::symbol_table::{SymbolTable, TypeSymTable};
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{FunctionType, StructType};
use crate::ast::statement::Statement::BlockStmt;
use crate::ast::statement::{Statement, TypedStmt, UntypedStmt};
use crate::compiler::codegen::BytecodeGen;

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

    pub fn analyze(&mut self) {
        self.record_top_level();
        self.record_consts();


        println!("{:?}", self.symbol_table);
        println!();
        println!();
        let resolved_root: TypedStmt = self.root.clone().resolve_type(&mut self.symbol_table);

        // TODO semantic analysis would probs be nice xd

        println!("{:#?}", resolved_root);

        let mut g = BytecodeGen::new();
        resolved_root.gen_bytecode(&mut g);

        println!();
        println!();
        println!("-------------------");
        println!();
        println!();

        for i in g.instructions {
            println!("{i:?}")
        }

    }


    /// record all top-level structs and functions which can be used everywhere
    fn record_top_level(&mut self) {
        if let BlockStmt{ body } = &self.root {
            for x in body {
                match x {
                    Statement::CommentStmt(_) | Statement::MultilineCommentStmt(_) => {},
                    Statement::VarDeclarationStmt {..} => {
                        // will resolve after functions and structs
                    },

                    Statement::FunctionStmt {name, params, return_type, .. } => {
                        self.symbol_table.record(name.clone(), FunctionType {
                            name: name.clone(),
                            params: params.iter().map(|p| p.param_type.clone()).collect(),
                            return_type: return_type.clone().boxed()
                        })
                    },

                    Statement::StructStmt {name, fields, body } => {
                        let mut children = HashMap::new();

                        for f in fields {
                            children.insert(f.name.clone(), f.param_type.clone());
                        }

                        for x in body {
                            match x {
                                Statement::FunctionStmt {name, return_type, params, .. } => {
                                    children.insert(name.clone(), AstType::FunctionType {
                                        name: name.clone(),
                                        return_type: return_type.clone().boxed(),
                                        params: params.iter().map(|p| p.clone().param_type).collect()
                                    });
                                },
                                _ => panic!("invalid statement inside struct {x:?}")
                            }
                        }

                        self.symbol_table.record(name.clone(), StructType {
                            name: name.clone(),
                            fields: fields.iter().map(|f| f.clone().param_type).collect(),
                            children
                        })
                    },
                    _ => panic!("Invalid statement {x:?}")
                }
            }

            return;
        }

        panic!("not a block statement? {:?}",self.root)
    }

    fn record_consts(&mut self) {
        if let BlockStmt{ body } = self.root.clone() {
            for x in body {
                match x {
                    Statement::VarDeclarationStmt {name, is_const, value, explicit_type} => {
                        if !is_const {
                            panic!("Top-level variables must be constant!")
                        }

                        let inferred_type = value.resolve_type(&mut self.symbol_table);

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

}