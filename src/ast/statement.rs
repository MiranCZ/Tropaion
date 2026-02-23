use std::fmt::Debug;
use crate::analysis::symbol_table::{SymbolTable, TypeSymTable};
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{Bool, StructType};
use crate::ast::expression::Expression;
use crate::ast::statement::Statement::{BlockStmt, CommentStmt, ExpressionStmt, FunctionStmt, IfStmt, MultilineCommentStmt, ReturnStmt, StructStmt, VarDeclarationStmt, WhileStmt};
use crate::lexer::token::SimpleToken::If;

pub type UntypedStmt = Statement<()>;
pub type TypedStmt = Statement<AstType>;

pub type StatementBlock<T> = Vec<Statement<T>>;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<T> {
    BlockStmt {
        body: StatementBlock<T>
    },
    ExpressionStmt(Expression<T>),
    VarDeclarationStmt {
        name: String,
        is_const: bool,
        value: Expression<T>,
        explicit_type: Option<AstType>
    },
    IfStmt {
        condition: Expression<T>,
        body: StatementBlock<T>,
        // either another `if_stmt` or `block_stmt`
        else_branch: Option<Box<Statement<T>>>
    },
    WhileStmt {
        condition: Expression<T>,
        body: StatementBlock<T>,
    },
    FunctionStmt {
        name: String,
        params: Vec<Parameter>,
        return_type: AstType,
        body: StatementBlock<T>
    },
    StructStmt {
        name: String,
        fields: Vec<Parameter>,
        body: StatementBlock<T>
    },
    ReturnStmt(Expression<T>),
    CommentStmt(String),
    MultilineCommentStmt(String)
}

impl <T> Statement<T> {

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: AstType
}


impl UntypedStmt {

    pub fn resolve_type(self, symbol_table: &mut TypeSymTable) -> TypedStmt {
        fn resolve_smt_block(body: StatementBlock<()>, symbol_table: &mut TypeSymTable) -> StatementBlock<AstType> {
            let mut typed_body = vec![];

            symbol_table.push();
            for x in body {
                typed_body.push(x.resolve_type(symbol_table));
            }
            symbol_table.pop();

            typed_body
        }

        match self {
            BlockStmt { body } => {
                BlockStmt {body: resolve_smt_block(body, symbol_table)}
            }
            ExpressionStmt(expr) => {
                let typed = expr.resolve_type(symbol_table);

                ExpressionStmt(typed)
            }
            VarDeclarationStmt { name, is_const, value, explicit_type } => {
                let typed_value = value.resolve_type(symbol_table);

                symbol_table.record(name.clone(), typed_value.get_type());

                let mut resolved_expl_type = None;

                if let Some(t) = explicit_type {
                    resolved_expl_type = Some(t.resolve_type(symbol_table));
                }

                VarDeclarationStmt {name, is_const, value: typed_value, explicit_type: resolved_expl_type}
            }
            IfStmt { condition, body, else_branch } => {
                let typed_condition = condition.resolve_type(symbol_table);

                if typed_condition.get_type() != Bool {
                    panic!("Condition should evaluate to a boolean! {:?}", typed_condition);
                }
                let mut typed_else_branch = None;
                if let Some(branch) = else_branch {
                    typed_else_branch = Some(branch.resolve_type(symbol_table).boxed());
                }

                IfStmt {condition: typed_condition, body: resolve_smt_block(body, symbol_table), else_branch: typed_else_branch}
            }
            WhileStmt { condition, body } => {
                let typed_condition = condition.resolve_type(symbol_table);

                if typed_condition.get_type() != Bool {
                    panic!("Condition should evaluate to a boolean! {:?}", typed_condition);
                }

                WhileStmt {condition: typed_condition, body: resolve_smt_block(body, symbol_table)}
            }
            FunctionStmt { name, params, return_type, body } => {
                let return_type = return_type.resolve_type(symbol_table);

                let mut resolved_params = vec![];

                for p in params.clone() {
                    resolved_params.push(Parameter{name: p.name, param_type: p.param_type.resolve_type(symbol_table)});
                }

                symbol_table.push();
                for p in params {
                    symbol_table.record(p.name, p.param_type);
                }

                let body = resolve_smt_block(body, symbol_table);

                symbol_table.pop();

                FunctionStmt {name, params: resolved_params, return_type, body}
            }
            StructStmt { name, fields, body } => {

                let mut resolved_fields = vec![];

                for p in fields {
                    resolved_fields.push(Parameter{name: p.name, param_type: p.param_type.resolve_type(symbol_table)});
                }

                symbol_table.push();
                let struct_type = symbol_table.get(name.clone()).unwrap();
                symbol_table.record(String::from("this"), struct_type.clone());

                if let StructType {children,..} = struct_type {
                    for p in children {
                        symbol_table.record(p.0, p.1.0);
                    }
                } else {
                    panic!("WTH type mismatch, got {struct_type:?}");
                }

                let body = resolve_smt_block(body, symbol_table);

                symbol_table.pop();

                StructStmt {name, fields: resolved_fields, body}
            }
            ReturnStmt(expr) => {
                ReturnStmt(expr.resolve_type(symbol_table))
            }
            CommentStmt(s) => {
                CommentStmt(s)
            }
            MultilineCommentStmt(s) => {
                MultilineCommentStmt(s)
            }
        }
    }

}