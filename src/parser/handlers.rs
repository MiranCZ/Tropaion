use crate::ast::ast_type::AstType;
use crate::ast::expression::Expression;
use crate::ast::statement::Statement;
use crate::error::parser_error::ParserError;
use crate::parser::binding_power::Bp;
use crate::parser::Parser;

pub type StatementHandler = fn(&mut Parser) -> Result<Box<dyn Statement>, ParserError>;
pub type NudHandler = fn(&mut Parser) -> Result<Box<dyn Expression>, ParserError>;
pub type LedHandler = fn(&mut Parser, Box<dyn Expression>, u32) -> Result<Box<dyn Expression>, ParserError>;

pub type TypeNudHandler = fn(&mut Parser) -> Result<Box<dyn AstType>, ParserError>;
pub type TypeLedHandler = fn(&mut Parser, Box<dyn AstType>, u32) -> Result<Box<dyn AstType>, ParserError>;


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