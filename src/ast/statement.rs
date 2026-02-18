use std::fmt::Debug;
use crate::ast::ast_type::AstType;
use crate::ast::expression::Expression;



pub type UntypedStmt = Statement<()>;
pub type TypedStmt = Statement<AstType>;

pub type StatementBlock<T> = Vec<Statement<T>>;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: AstType
}
