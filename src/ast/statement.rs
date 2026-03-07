use crate::analysis::symbol_table::TypeSymTable;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::{Bool, StructType};
use crate::ast::expression::Expression;
use crate::ast::statement::Statement::{BlockStmt, CommentStmt, ExpressionStmt, FunctionStmt, IfStmt, MultilineCommentStmt, ReturnStmt, StructStmt, VarDeclarationStmt, WhileStmt};
use crate::error::analysis_error::AnalysisError;
use crate::error::runtime_error::ValueTypeVariant;
use std::fmt::Debug;
use crate::error::context::ErrorContext;
use crate::util::spanned::Spanned;

pub type UntypedStmt = Spanned<Statement<()>>;
pub type TypedStmt = Spanned<Statement<TypeEntry>>;

pub type StatementBlock<T> = Vec<Spanned<Statement<T>>>;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<T> {
    BlockStmt {
        body: StatementBlock<T>
    },
    ExpressionStmt(Spanned<Expression<T>>),
    VarDeclarationStmt {
        name: String,
        is_const: bool,
        value: Spanned<Expression<T>>,
        explicit_type: Option<TypeEntry>
    },
    IfStmt {
        condition: Spanned<Expression<T>>,
        body: StatementBlock<T>,
        // either another `if_stmt` or `block_stmt`
        else_branch: Option<Box<Spanned<Statement<T>>>>
    },
    WhileStmt {
        condition: Spanned<Expression<T>>,
        body: StatementBlock<T>,
    },
    FunctionStmt {
        name: String,
        params: Vec<Parameter>,
        return_type: TypeEntry,
        body: StatementBlock<T>
    },
    StructStmt {
        name: String,
        fields: Vec<Parameter>,
        body: StatementBlock<T>
    },
    ReturnStmt(Spanned<Expression<T>>),
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
    pub param_type: TypeEntry
}


impl UntypedStmt {

    pub fn resolve_type(self,registry: &mut TypeRegistry ,symbol_table: &mut TypeSymTable) -> Result<TypedStmt, ErrorContext<AnalysisError>> {
        fn resolve_smt_block(body: StatementBlock<()>,registry: &mut TypeRegistry, symbol_table: &mut TypeSymTable) -> Result<StatementBlock<TypeEntry>, ErrorContext<AnalysisError>> {
            let mut typed_body = vec![];

            symbol_table.push();
            for x in body {
                typed_body.push(x.resolve_type(registry, symbol_table)?);
            }
            symbol_table.pop();

            Ok(typed_body)
        }

        let ctx = |err| {
           ErrorContext::of(err, self.span)
        };

        let typed_stmt = match self.node {
            BlockStmt { body } => {
                BlockStmt {body: resolve_smt_block(body,registry, symbol_table)?}
            }
            ExpressionStmt(expr) => {
                let typed = expr.resolve_type(registry, symbol_table)?;

                ExpressionStmt(typed)
            }
            VarDeclarationStmt { name, is_const, value, explicit_type } => {
                let mut typed_value = value.resolve_type(registry, symbol_table)?;

                symbol_table.record(name.clone(), typed_value.get_type());

                let mut resolved_expl_type = None;

                if let Some(t) = explicit_type {
                    t.resolve_type(registry, symbol_table)?;

                    if let Some(new_t) = t.get(registry).get_assign_result(typed_value.get_type().get(registry), registry) {
                        typed_value.set_type(registry, new_t);
                    } else {
                        return Err(ctx(AnalysisError::illegal_type_assignment(t, typed_value.get_type(), registry)));
                    }

                    resolved_expl_type = Some(t);
                }


                VarDeclarationStmt {name, is_const, value: typed_value, explicit_type: resolved_expl_type}
            }
            IfStmt { condition, body, else_branch } => {
                let typed_condition = condition.resolve_type(registry, symbol_table)?;

                if typed_condition.get_type().get(registry) != Bool {
                    return Err(ctx(AnalysisError::type_mismatch(ValueTypeVariant::Bool, typed_condition.get_type(), registry)));
                }
                let mut typed_else_branch = None;
                if let Some(branch) = else_branch {
                    typed_else_branch = Some(branch.resolve_type(registry, symbol_table)?.boxed());
                }

                IfStmt {condition: typed_condition, body: resolve_smt_block(body, registry, symbol_table)?, else_branch: typed_else_branch}
            }
            WhileStmt { condition, body } => {
                let typed_condition = condition.resolve_type(registry, symbol_table)?;

                if typed_condition.get_type().get(registry) != Bool {
                    return Err(ctx(AnalysisError::type_mismatch(ValueTypeVariant::Bool, typed_condition.get_type(), registry)));
                }

                WhileStmt {condition: typed_condition, body: resolve_smt_block(body, registry, symbol_table)?}
            }
            FunctionStmt { name, mut params, return_type, body } => {
                return_type.resolve_type(registry, symbol_table)?;


                for p in params.iter_mut() {
                    p.param_type.resolve_type(registry, symbol_table)?;
                }

                symbol_table.push();

                symbol_table.record_return_type(return_type);

                for p in params.clone() {
                    symbol_table.record(p.name, p.param_type);
                }

                let body = resolve_smt_block(body, registry, symbol_table)?;

                symbol_table.pop();

                FunctionStmt {name, params, return_type, body}
            }
            StructStmt { name, mut fields, body } => {
                for p in fields.iter_mut() {
                    p.param_type.resolve_type(registry, symbol_table)?;
                }

                symbol_table.push();
                let struct_type = symbol_table.get(name.clone()).unwrap();
                symbol_table.record(String::from("this"), struct_type.clone());

                if let StructType {children,..} = struct_type.get(registry) {
                    for p in children {
                        symbol_table.record_with_info(p.0, p.1.0, true);
                    }
                } else {
                    return Err(ctx(AnalysisError::type_mismatch(ValueTypeVariant::Struct, struct_type, registry)));
                }

                let body = resolve_smt_block(body, registry, symbol_table)?;

                symbol_table.pop();

                StructStmt {name, fields, body}
            }
            ReturnStmt(expr) => {
                let mut typed_expr = expr.resolve_type(registry, symbol_table)?;

                let return_type = symbol_table.get_return_type();

                let return_type = if let Some(r) = return_type {
                    r
                } else {
                    return Err(ctx(AnalysisError::DanglingReturn));
                };

                if let Some(r) = return_type.get(registry).get_assign_result(typed_expr.get_type().get(registry), registry) {
                    typed_expr.set_type(registry, r);
                } else {
                    return Err(ctx(AnalysisError::illegal_type_assignment(return_type, typed_expr.get_type(), registry)));
                }

                ReturnStmt(typed_expr)
            }
            CommentStmt(s) => {
                CommentStmt(s)
            }
            MultilineCommentStmt(s) => {
                MultilineCommentStmt(s)
            }
        };
        
        Ok(Spanned::of(typed_stmt, self.span))
    }

}