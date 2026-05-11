use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::*;
use crate::error::context::ErrorContext;
use crate::error::parser_error::ParserError;
use crate::lexer::token::SimpleToken::{CloseBracket, Question, TwoQuestion};
use crate::lexer::token::SimpleToken;
use crate::parser::binding_power::{Bp, DEFAULT};
use crate::parser::handlers::ReturnedType;
use crate::parser::Parser;

pub fn parse_type(registry: &mut TypeRegistry,parser: &mut Parser, binding_power: Bp) -> ReturnedType {
    let token = parser.peek()?;

    let nud_fn = token.type_nud(&parser.type_lookup);

    if nud_fn.is_none() {
        return Err(ErrorContext::of(ParserError::NUDMissing(token), parser.current_span()));
    }

    let nud_fn = nud_fn.unwrap();

    let mut left = nud_fn(registry, parser)?;

    loop {
        let token = parser.peek()?;

        let led_info = token.type_led(&parser.type_lookup);

        if led_info.is_none() {
            return Ok(left);
        }

        let led_info = led_info.unwrap();

        let rbp = led_info.rbp;
        let lbp = led_info.lbp;
        let led_fn = led_info.type_handler;

        if lbp < binding_power {
            return Ok(left);
        }

        left = led_fn(registry, parser, left, rbp)?;
    }
}

pub fn parse_reference_type(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedType {
    parser.expect_next(SimpleToken::Ampersand)?;

    let expr = parse_type(registry, parser, DEFAULT.rbp)?;

    Ok(registry.register(ReferenceType { underlying: expr }))
}

pub fn parse_array_type(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedType {
    parser.expect_next(SimpleToken::OpenSquare)?;

    let expr = parse_type(registry, parser, DEFAULT.rbp)?;

    parser.expect_next(SimpleToken::CloseSquare)?;

    Ok(registry.register(ArrayType{
        underlying: expr,
    }))

}

pub fn parse_tuple_type(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedType {
    parser.expect_next(SimpleToken::OpenBracket)?;

    let mut result = vec![];
    loop {
        let expr = parse_type(registry, parser, DEFAULT.rbp)?;

        result.push(expr);

        if parser.consume_if_next(CloseBracket)? {
            break;
        }

        parser.expect_next(SimpleToken::Comma)?;
    }

    Ok(registry.register(TupleType(result)))
}

pub fn parse_nullable_type(registry: &mut TypeRegistry,parser: &mut Parser, left: TypeEntry, _bp: u32) -> ReturnedType {
    parser.expect_next(Question)?;

    Ok(registry.register(NullableType {
        underlying: left
    }))
}


// throwing an error is the analyzers job
pub fn parse_double_nullable_type(registry: &mut TypeRegistry,parser: &mut Parser, left: TypeEntry, bp: u32) -> ReturnedType {
    parser.expect_next(TwoQuestion)?;

    let nullable = registry.register(NullableType {
        underlying: left
    });

    Ok(registry.register(NullableType {
        underlying: nullable
    }))
}