use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::*;
use crate::error::parser_error::ParserError;
use crate::lexer::token::{SimpleToken, Token};
use crate::lexer::token::SimpleToken::{CloseBracket, Question};
use crate::parser::binding_power::{Bp, DEFAULT};
use crate::parser::handlers::ReturnedType;
use crate::parser::Parser;

pub fn parse_type(parser: &mut Parser, binding_power: Bp) -> ReturnedType {
    let token = parser.peek()?;

    let nud_fn = token.type_nud(&parser.type_lookup);

    if nud_fn.is_none() {
        return Err(ParserError::NUDMissing(token));
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

pub fn parse_reference_type(parser: &mut Parser) -> ReturnedType {
    parser.expect_next(SimpleToken::Ampersand)?;

    let expr = parse_type(parser, DEFAULT)?;

    Ok(ReferenceType { underlying: expr.boxed() })
}

pub fn parse_array_type(parser: &mut Parser) -> ReturnedType {
    parser.expect_next(SimpleToken::OpenSquare)?;

    let expr = parse_type(parser, DEFAULT)?;

    parser.expect_next(SimpleToken::Semicolon)?;

    let count = parser.expect_next_int()?;

    parser.expect_next(SimpleToken::CloseSquare)?;

    Ok(ArrayType{
        underlying: expr.boxed(),
        count: count as u32
    })

}

pub fn parse_tuple_type(parser: &mut Parser) -> ReturnedType {
    parser.expect_next(SimpleToken::OpenBracket)?;

    let mut result = vec![];
    loop {
        let expr = parse_type(parser, DEFAULT)?;

        result.push(expr);

        if parser.consume_if_next(CloseBracket)? {
            break;
        }

        parser.expect_next(SimpleToken::Comma)?;
    }

    Ok(TupleType(result))
}

pub fn parse_nullable_type(parser: &mut Parser, left: AstType, _bp: u32) -> ReturnedType {
    parser.expect_next(Question)?;

    Ok(NullableType {
        underlying: left.boxed()
    })
}