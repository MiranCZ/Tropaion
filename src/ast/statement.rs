use std::fmt::Debug;
use crate::ast::expression::Expression;


pub trait Statement : Debug{
}

#[derive(Debug)]
pub struct BlockStmt {
    pub body: Vec<Box<dyn Statement>>
}

#[derive(Debug)]
pub struct ExpressionStmt(pub Box<dyn Expression>);

#[derive(Debug)]
pub struct VarDeclarationStmt {
    pub name: String,
    pub is_const: bool,
    pub value: Box<dyn Expression>
}

impl Statement for VarDeclarationStmt{}
impl Statement for ExpressionStmt{}
impl Statement for BlockStmt{}