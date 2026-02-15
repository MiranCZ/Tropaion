use crate::ast::expression::{BinaryExpr, Expression};
use crate::error::parser_error::ParserError;
use crate::lexer::token::Token;
use crate::lexer::token::Token::SimpleTokenType;
use crate::parser::binding_power::Bp;
use crate::parser::Parser;

pub fn parse_expression(parser: &mut Parser, binding_power: Bp) -> Result<Box<dyn Expression>, ParserError> {
    let token = parser.peek()?;

    let nud_fn = token.nud(&parser.lookup);

    if nud_fn.is_none() {
        return Err(ParserError::NUDMissing);
    }

    let nud_fn = nud_fn.unwrap();

    let mut left = nud_fn(parser)?;

    loop {
        let token = parser.peek()?;

        let led_info = token.led(&parser.lookup);

        if led_info.is_none() {
            // !("LED not implemented for {token:?}");
            return Ok(left);
        }

        let led_info = led_info.unwrap();

        let rbp = led_info.rbp;
        let lbp = led_info.lfb;
        let led_fn = led_info.handler;

        if lbp < binding_power {
            return Ok(left);
        }

        left = led_fn(parser, left, rbp)?;
    }
}


pub fn parse_binary_expr(parser: &mut Parser, left: Box<dyn Expression>, binding_power: Bp) -> Result<Box<dyn Expression>, ParserError> {
    let operator = parser.next()?;

    if let SimpleTokenType(t) = operator {
        let right = parse_expression(parser, binding_power)?;

        return Ok(Box::new(BinaryExpr { left, operator: t, right }));
    }

    panic!("uh oh")
}