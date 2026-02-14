use std::fmt::Debug;
use crate::lexer::token::SimpleToken;

pub trait Expression : Debug {
}

#[derive(Debug)]
pub struct IntLiteralExpr(pub i32);
#[derive(Debug)]
pub struct FloatLiteralExpr(pub f32);
#[derive(Debug)]
pub struct StringLiteralExpr(pub String);

#[derive(Debug)]
pub struct IdentifierExpr(pub String);
impl Expression for IdentifierExpr {}
impl Expression for IntLiteralExpr {}
impl Expression for StringLiteralExpr {}
impl Expression for FloatLiteralExpr {}
impl Expression for BinaryExpr {}

#[derive(Debug)]
pub struct BinaryExpr {
    pub left: Box<dyn Expression>,
    pub operator: SimpleToken,
    pub right: Box<dyn Expression>
}

