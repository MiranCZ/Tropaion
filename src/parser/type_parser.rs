use crate::ast::ast_type::{ArrayType, AstType};
use crate::error::parser_error::ParserError;
use crate::lexer::token::{SimpleToken, Token};
use crate::parser::binding_power::{Bp, DEFAULT};
use crate::parser::Parser;

pub fn parse_type(parser: &mut Parser, binding_power: Bp) -> Result<Box<dyn AstType>, ParserError> {
    let token = parser.peek()?;

    let nud_fn = token.type_nud(&parser.type_lookup);

    if nud_fn.is_none() {
        return Err(ParserError::NUDMissing);
    }

    let nud_fn = nud_fn.unwrap();

    let mut left = nud_fn(parser)?;

    loop {
        let token = parser.peek()?;

        let led_info = token.type_led(&parser.type_lookup);

        if led_info.is_none() {
            return Ok(left);
        }

        let led_info = led_info.unwrap();

        let rbp = led_info.rbp;
        let lbp = led_info.lfb;
        let led_fn = led_info.type_handler;

        if lbp < binding_power {
            return Ok(left);
        }

        left = led_fn(parser, left, rbp)?;
    }
}

pub fn parse_array_type(parser: &mut Parser) -> Result<Box<dyn AstType>, ParserError> {
    parser.expect_next(Token::SimpleTokenType(SimpleToken::OpenSquare))?;

    let expr = parse_type(parser, DEFAULT)?;

    parser.expect_next(Token::SimpleTokenType(SimpleToken::Semicolon))?;

    let next = parser.next()?;

    if let Token::NumberIntLiteral(count) = next {
        parser.expect_next(Token::SimpleTokenType(SimpleToken::CloseSquare))?;

        return Ok(Box::new(ArrayType{
            underlying: expr,
            count: count as u32
        }));
    }

    Err(ParserError::UnexpectedToken)
}