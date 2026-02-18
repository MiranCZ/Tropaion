use std::string::String;
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{Bool, Float, Int, StringType};
use crate::ast::expression::Expression::{AssignExpr, BinaryExpr, BoolLiteralExpr, CallExpr, DecrementExpr, FloatLiteralExpr, IdentifierExpr, IncrementExpr, IntLiteralExpr, MemberExpr, PrefixExpr, StringLiteralExpr, TupleExpr};
use crate::lexer::token::SimpleToken;

pub type UntypedExpr = Expression<()>;
pub type TypedExpr = Expression<AstType>;

#[derive(Debug, PartialEq)]
pub enum Expression<T> {
    BoolLiteralExpr(bool),
    IntLiteralExpr(i32),
    FloatLiteralExpr(f32),
    StringLiteralExpr(String),
    IdentifierExpr(T, String),
    IncrementExpr(T, Box<Expression<T>>),
    DecrementExpr(T, Box<Expression<T>>),
    PrefixExpr {
        t: T,
        operator: SimpleToken,
        expr: Box<Expression<T>>
    },
    BinaryExpr {
        t: T,
        left: Box<Expression<T>>,
        operator: SimpleToken,
        right: Box<Expression<T>>
    },
    AssignExpr {
        t: T,
        assignee: Box<Expression<T>>,
        operator: SimpleToken,
        value: Box<Expression<T>>
    },
    TupleExpr {
        t: T,
        values: Vec<Expression<T>>
    },
    MemberExpr {
        t: T,
        member: Box<Expression<T>>,
        property: Box<Expression<T>>
    },
    CallExpr {
        t: T,
        func: Box<Expression<T>>,
        args: Vec<Expression<T>>
    }
}

impl <T> Expression<T> {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl TypedExpr {
    pub fn get_type(&self) -> AstType {
        match self {
            BoolLiteralExpr(_) => Bool,
            IntLiteralExpr(_) => Int,
            FloatLiteralExpr(_) => Float,
            StringLiteralExpr(_) => StringType,
            IdentifierExpr(t, _) => t.clone(),
            IncrementExpr(t, _) => t.clone(),
            DecrementExpr(t, _) => t.clone(),
            PrefixExpr {t, .. } => t.clone(),
            BinaryExpr {t, .. } => t.clone(),
            AssignExpr {t, .. } => t.clone(),
            TupleExpr {t, .. } => t.clone(),
            MemberExpr {t, .. } => t.clone(),
            CallExpr {t, .. } => t.clone()
        }
    }
}

pub fn bool(b: bool) -> UntypedExpr {
    BoolLiteralExpr(b)
}

pub fn int(i: i32) -> UntypedExpr {
    IntLiteralExpr(i)
}

pub fn float(f: f32) -> UntypedExpr {
    FloatLiteralExpr(f)
}

pub fn string(s: String) -> UntypedExpr {
    StringLiteralExpr(s)
}

pub fn identifier(identifier: String) -> UntypedExpr {
    IdentifierExpr((), identifier)
}

pub fn increment(expr: UntypedExpr) -> UntypedExpr {
    IncrementExpr((), expr.boxed())
}

pub fn decrement(expr: UntypedExpr) -> UntypedExpr {
    DecrementExpr((), expr.boxed())
}

pub fn prefix(operator: SimpleToken, expr: UntypedExpr) -> UntypedExpr {
    PrefixExpr {
        t: (),
        operator,
        expr: expr.boxed()
    }
}

pub fn binary(left: UntypedExpr, operator: SimpleToken, right: UntypedExpr) -> UntypedExpr {
    BinaryExpr {
        t: (),
        left: left.boxed(),
        operator,
        right: right.boxed()
    }
}

pub fn assign(assignee: UntypedExpr, operator: SimpleToken, value: UntypedExpr) -> UntypedExpr {
    AssignExpr {
        t: (),
        assignee: assignee.boxed(),
        operator,
        value: value.boxed()
    }
}

pub fn tuple(values: Vec<UntypedExpr>) -> UntypedExpr {
    TupleExpr {
        t: (),
        values
    }
}

pub fn member(member: UntypedExpr, property: UntypedExpr) -> UntypedExpr {
    MemberExpr {
        t: (),
        member: member.boxed(),
        property: property.boxed()
    }
}

pub fn call(func: UntypedExpr, args: Vec<UntypedExpr>) -> UntypedExpr {
    CallExpr {
        t: (),
        func: func.boxed(),
        args
    }
}
