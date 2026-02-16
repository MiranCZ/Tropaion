use std::fmt::Debug;
use tropaion_derive::{ast_type, statement};
use crate::ast::ast_type::AstType;
use crate::ast::expression::Expression;


pub trait Statement : Debug{
}

#[statement]
pub struct BlockStmt {
    pub body: Vec<Box<dyn Statement>>
}

#[statement]
pub struct ExpressionStmt(pub Box<dyn Expression>);

#[statement]
pub struct VarDeclarationStmt {
    pub name: String,
    pub is_const: bool,
    pub value: Box<dyn Expression>,
    pub explicit_type: Option<Box<dyn AstType>>
}

#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub param_type: Box<dyn AstType>
}

#[statement]
pub struct FunctionStmt {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Box<dyn AstType>>,
    pub body: BlockStmt
}

#[statement]
pub struct ReturnStmt(pub Box<dyn Expression>);

#[statement]
pub struct CommentStmt(pub String);

#[statement]
pub struct MultilineCommentStmt(pub String);