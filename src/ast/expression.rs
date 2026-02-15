use std::fmt::Debug;
use tropaion_derive::expression;
use crate::lexer::token::SimpleToken;

pub trait Expression : Debug {
}

#[expression]
pub struct IntLiteralExpr(pub i32);

#[expression]
pub struct FloatLiteralExpr(pub f32);

#[expression]
pub struct StringLiteralExpr(pub String);

#[expression]
pub struct IdentifierExpr(pub String);

#[expression]
pub struct BinaryExpr {
    pub left: Box<dyn Expression>,
    pub operator: SimpleToken,
    pub right: Box<dyn Expression>
}

