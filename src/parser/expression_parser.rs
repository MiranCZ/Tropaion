use crate::ast::expression;
use crate::ast::expression::{Expression, UntypedExpr};
use crate::ast::expression::Expression::*;
use crate::ast::statement::Parameter;
use crate::error::parser_error::ParserError;
use crate::lexer::token::SimpleToken::{CloseBracket, CloseSquare, Comma, Dot, False, OpenBracket, True};
use crate::lexer::token::Token;
use crate::lexer::token::Token::SimpleTokenType;
use crate::parser::binding_power::{Bp, ASSIGNMENT, DEFAULT, UNARY};
use crate::parser::handlers::ReturnedExpression;
use crate::parser::Parser;

pub fn parse_expression(parser: &mut Parser, binding_power: Bp) -> ReturnedExpression {
    let token = parser.peek()?;

    let nud_fn = token.nud(&parser.lookup);

    if nud_fn.is_none() {
        return Err(ParserError::NUDMissing(token));
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

pub fn parse_prefix_expr(parser: &mut Parser) -> ReturnedExpression {
    let operator = parser.expect_next_simple()?;

    let expr = parse_expression(parser, UNARY)?;

    Ok(expression::prefix(operator, expr))
}

pub fn parse_binary_expr(parser: &mut Parser, left: UntypedExpr, binding_power: Bp) -> ReturnedExpression {
    let operator = parser.expect_next_simple()?;

    let right = parse_expression(parser, binding_power)?;

    Ok(expression::binary(left, operator, right))
}

pub fn parse_bool_literal_expr(parser: &mut Parser) -> ReturnedExpression {
    if parser.consume_if_next(True)? {
        return Ok(BoolLiteralExpr(true));
    }
    if parser.consume_if_next(False)? {
        return Ok(BoolLiteralExpr(false));
    }
    
    panic!("Invalid call")
}

pub fn parse_increment_expr(parser: &mut Parser, left: UntypedExpr, _bp: Bp) -> ReturnedExpression {
    parser.next()?;
    Ok(expression::increment(left))
}

pub fn parse_decrement_expr(parser: &mut Parser, left: UntypedExpr, _bp: Bp) -> ReturnedExpression {
    parser.next()?;
    Ok(expression::decrement(left))
}

pub fn parse_parenthesis_expr(parser: &mut Parser) -> ReturnedExpression {
    parser.expect_next(OpenBracket)?;

    let expr = parse_expression(parser, DEFAULT)?;

    // we are defining a tuple
    if parser.consume_if_next(Comma)? {
        let mut values = vec![];
        values.push(expr);

        loop {
            let value = parse_expression(parser, DEFAULT)?;

            values.push(value);

            if !parser.consume_if_next(Comma)? {
                parser.expect_next(CloseBracket)?;
                break;
            }
        }

        return Ok(expression::tuple(values));
    }

    parser.expect_next(CloseBracket)?;

    Ok(expr)
}

pub fn parse_member_expr(parser: &mut Parser, left: UntypedExpr, binding_power: Bp) -> ReturnedExpression {
    parser.expect_next(Dot)?;
    
    let right = parse_expression(parser, binding_power)?;

    Ok(expression::member(left, right))
}

pub fn parse_assignment_expr(parser: &mut Parser, left: UntypedExpr, binding_power: Bp) -> ReturnedExpression {
    let operator = parser.expect_next_simple()?;
    
    let value = parse_expression(parser, binding_power)?;
   
    Ok(expression::assign(left, operator, value))
}

pub fn parse_call_expr(parser: &mut Parser, left: UntypedExpr, binding_power: Bp) -> ReturnedExpression {
    parser.expect_next(OpenBracket)?;
    
    let mut args = vec![];

    while !parser.consume_if_next(CloseBracket)? {
        args.push(parse_expression(parser, ASSIGNMENT)?);
        
        if !parser.consume_if_next(Comma)? {
            parser.expect_next(CloseBracket)?;
            break;
        }
    }

    Ok(expression::call(left, args))
}