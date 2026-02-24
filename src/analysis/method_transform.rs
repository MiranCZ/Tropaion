use crate::analysis::symbol_table::{SymbolTable, TypeSymTable};
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::SymbolType;
use crate::ast::statement::Statement::{BlockStmt, FunctionStmt, IfStmt, StructStmt, WhileStmt};
use crate::ast::statement::{Parameter, TypedStmt};

impl TypedStmt {

    pub fn transform_methods(self, symbol_table: &TypeSymTable) -> TypedStmt {
        self._transform_methods(symbol_table, &None)
    }

    fn _transform_methods(self,symbol_table: &TypeSymTable, inside_struct: &Option<AstType>) -> TypedStmt {
        match self {
            BlockStmt {body,  .. } => {
                let mut resolved = vec![];

                for b in body {
                    resolved.push(b._transform_methods(symbol_table, inside_struct));
                }

                BlockStmt {body: resolved}
            }
            IfStmt {condition, body, else_branch } => {
                let mut resolved = vec![];

                for b in body {
                    resolved.push(b._transform_methods(symbol_table , inside_struct));
                }

                let mut resolved_else = None;

                if let Some(v) = else_branch {
                    resolved_else = Some(v._transform_methods(symbol_table ,inside_struct).boxed());
                }


                IfStmt {condition, body: resolved, else_branch: resolved_else}
            }
            WhileStmt {condition, body, .. } => {
                let mut resolved = vec![];

                for b in body {
                    resolved.push(b._transform_methods(symbol_table ,inside_struct));
                }

                WhileStmt {condition, body: resolved}
            }
            StructStmt { name, fields, body } => {
                let mut resolved = vec![];

                for b in body {
                    resolved.push(b._transform_methods(symbol_table, &Some(symbol_table.get(name.clone()).unwrap())));
                }

                StructStmt {body: resolved, name, fields}
            }
            FunctionStmt { name, params, return_type, body } => {
                let mut mutated = vec![];

                for p in params {
                    mutated.push(p);
                }

                if let Some(t) = inside_struct {
                    mutated.insert(0, Parameter{name: "this".to_string(), param_type: t.clone()});
                }

                FunctionStmt {name, params: mutated, return_type, body}
            }

            _ => self
        }
    }

}