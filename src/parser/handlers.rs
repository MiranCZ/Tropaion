use crate::ast::expression::Expression;
use crate::ast::statement::Statement;
use crate::parser::binding_power::Bp;
use crate::parser::Parser;

pub type StatementHandler = fn(&mut Parser) -> Box<dyn Statement>;
pub type NudHandler = fn(&mut Parser) -> Box<dyn Expression>;
pub type LedHandler = fn(&mut Parser, Box<dyn Expression>, u32) -> Box<dyn Expression>;


#[derive(Clone, Copy)]
pub struct LedInfo {
    pub handler: LedHandler,
    pub rbp: Bp,
    pub lfb: Bp
}