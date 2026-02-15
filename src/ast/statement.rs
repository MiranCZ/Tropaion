use std::fmt::Debug;
use tropaion_derive::statement;
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
    pub value: Box<dyn Expression>
}
