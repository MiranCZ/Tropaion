use crate::ast::ast_type::AstType;
use crate::ast::expression::Expression;
use crate::ast::statement::Statement;
use crate::error::parser_error::ParserError;
use crate::parser::binding_power::Bp;
use crate::parser::Parser;


pub type ReturnedStatement = Result<Statement, ParserError>;
pub type ReturnedExpression = Result<Expression, ParserError>;
pub type ReturnedType = Result<AstType, ParserError>;


pub type StatementHandler = fn(&mut Parser) -> ReturnedStatement;
pub type NudHandler = fn(&mut Parser) -> ReturnedExpression;
pub type LedHandler = fn(&mut Parser, Expression, u32) -> ReturnedExpression;

pub type TypeNudHandler = fn(&mut Parser) -> ReturnedType;
pub type TypeLedHandler = fn(&mut Parser, AstType, u32) -> ReturnedType;


#[derive(Clone, Copy)]
pub struct LedInfo {
    pub handler: LedHandler,
    pub rbp: Bp,
    pub lfb: Bp
}

#[derive(Clone, Copy)]
pub struct TypeLedInfo {
    pub type_handler: TypeLedHandler,
    pub rbp: Bp,
    pub lfb: Bp
}