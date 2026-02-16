use std::fmt::Debug;
use crate::ast::ast_type::AstType;
use crate::ast::expression::Expression;


pub type StatementBlock = Vec<Statement>;

#[derive(Debug, PartialEq)]
pub enum Statement {
    BlockStmt {
        body: StatementBlock
    },
    ExpressionStmt(Expression),
    VarDeclarationStmt {
        name: String,
        is_const: bool,
        value: Expression,
        explicit_type: Option<AstType>
    },
    FunctionStmt {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<AstType>,
        body: StatementBlock
    },
    ReturnStmt(Expression),
    CommentStmt(String),
    MultilineCommentStmt(String)
}


#[derive(Debug, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: AstType
}
