use crate::lexer::token::SimpleToken;

#[derive(Debug, PartialEq)]
pub enum Expression {
    BoolLiteralExpr(bool),
    IntLiteralExpr(i32),
    FloatLiteralExpr(f32),
    StringLiteralExpr(String),
    IdentifierExpr(String),
    IncrementExpr(Box<Expression>),
    DecrementExpr(Box<Expression>),
    PrefixExpr {
        operator: SimpleToken,
        expr: Box<Expression>
    },
    BinaryExpr {
        left: Box<Expression>,
        operator: SimpleToken,
        right: Box<Expression>
    },
    AssignExpr {
        assignee: Box<Expression>,
        operator: SimpleToken,
        value: Box<Expression>
    },
    TupleExpr {
        values: Vec<Expression>
    }
}

impl Expression {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}