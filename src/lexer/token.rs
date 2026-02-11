use crate::lexer::symbols::{Keywords, Operators};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Token {
    Identifier(String),
    NumberIntLiteral(i32),
    NumberFloatLiteral(f32),
    StringLiteral(String),
    Keyword(Keywords),
    Separator(char),
    Operator(Operators),
    Comment(String),
    EOF
}