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
    IfStmt {
        condition: Expression,
        body: StatementBlock,
        // either another `if_stmt` or `block_stmt`
        else_branch: Option<Box<Statement>>  
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

impl Statement {
    
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
    
}

#[derive(Debug, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: AstType
}
