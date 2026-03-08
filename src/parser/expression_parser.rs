use crate::analysis::type_registry::TypeRegistry;
use crate::ast::expression;
use crate::ast::expression::{Expression, UntypedExpr};
use crate::ast::expression::Expression::*;
use crate::ast::statement::Parameter;
use crate::error::parser_error::ParserError;
use crate::lexer::token::SimpleToken::{CloseBracket, CloseSquare, Comma, Dot, False, Null, OpenBracket, OpenSquare, QuestionDot, True, TwoExcl};
use crate::lexer::token::Token;
use crate::lexer::token::Token::SimpleTokenType;
use crate::parser::binding_power::{Bp, ASSIGNMENT, COMMA, DEFAULT, UNARY};
use crate::parser::handlers::ReturnedExpression;
use crate::parser::Parser;
use crate::{spanned, spanned_led};
use crate::error::context::ErrorContext;

pub fn parse_expression(registry: &mut TypeRegistry, parser: &mut Parser, binding_power: Bp) -> ReturnedExpression {
    let token = parser.peek()?;

    let nud_fn = token.nud(&parser.lookup);

    if nud_fn.is_none() {
        return Err(ErrorContext::of(ParserError::NUDMissing(token), parser.current_span()));
    }

    let nud_fn = nud_fn.unwrap();

    let mut left = nud_fn(registry, parser)?;

    loop {
        let token = parser.peek()?;

        let led_info = token.led(&parser.lookup);

        if led_info.is_none() {
            // !("LED not implemented for {token:?}");
            return Ok(left);
        }

        let led_info = led_info.unwrap();

        let rbp = led_info.rbp;
        let lbp = led_info.lbp;
        let led_fn = led_info.handler;

        if lbp < binding_power {
            return Ok(left);
        }

        left = led_fn(registry,parser, left, rbp)?;
    }
}

pub fn parse_prefix_expr(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedExpression {
    spanned!(parser, {
        let operator = parser.expect_next_simple()?;

        let expr = parse_expression(registry,parser, UNARY.rbp)?;

        expression::prefix(operator, expr)
    })
}

pub fn parse_binary_expr(registry: &mut TypeRegistry,parser: &mut Parser, left: UntypedExpr, binding_power: Bp) -> ReturnedExpression {
    spanned_led!(parser, left, {
        let operator = parser.expect_next_simple()?;

        let right = parse_expression(registry,parser, binding_power)?;

        expression::binary(left, operator, right)
    })
}

pub fn parse_bool_literal_expr(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedExpression {
    spanned!(parser, {
        if parser.consume_if_next(True)? {
            BoolLiteralExpr((), true)
        } else if parser.consume_if_next(False)? {
            BoolLiteralExpr((), false)
        } else {
            panic!("Invalid call")
        }
    })
}


pub fn parse_null_expr(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedExpression {
    spanned!(parser, {
        parser.expect_next(Null)?;

        NullLiteralExpr(())
    })
}

pub fn parse_array_expr(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedExpression {
    spanned!(parser, {
        parser.expect_next(OpenSquare)?;

        let mut values = vec![];

        while !parser.consume_if_next(CloseSquare)? {
            values.push(parse_expression(registry, parser, COMMA.rbp)?);

            if !parser.consume_if_next(Comma)? {
                parser.expect_next(CloseSquare)?;
                break;
            }
        }

        expression::array_literal(values)
    })
}

pub fn parse_increment_expr(registry: &mut TypeRegistry,parser: &mut Parser, left: UntypedExpr, _bp: Bp) -> ReturnedExpression {
    spanned_led!(parser, left, {
        parser.next()?;
        expression::increment(left)
    })
}

pub fn parse_decrement_expr(registry: &mut TypeRegistry,parser: &mut Parser, left: UntypedExpr, _bp: Bp) -> ReturnedExpression {
    spanned_led!(parser, left, {
        parser.next()?;
        expression::decrement(left)
    })
}

pub fn parse_parenthesis_expr(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedExpression {
    spanned!(parser, {
        parser.expect_next(OpenBracket)?;

        let expr = parse_expression(registry,parser, DEFAULT.rbp)?;

        // we are defining a tuple
        if parser.consume_if_next(Comma)? {
            let mut values = vec![];
            values.push(expr);

            loop {
                let value = parse_expression(registry,parser, DEFAULT.rbp)?;

                values.push(value);

                if !parser.consume_if_next(Comma)? {
                    parser.expect_next(CloseBracket)?;
                    break;
                }
            }

            expression::tuple(values)
        } else {
            parser.expect_next(CloseBracket)?;

            expr.node
        }
    })
}

pub fn parse_member_expr(registry: &mut TypeRegistry,parser: &mut Parser, left: UntypedExpr, binding_power: Bp) -> ReturnedExpression {
    spanned_led!(parser, left, {
        let mut null_safe = false;
        if parser.consume_if_next(QuestionDot)? {
            null_safe = true;
        } else {
            parser.expect_next(Dot)?;
        }

        let right = parse_expression(registry,parser, binding_power)?;

        expression::member(left, right, null_safe)
    })
}

pub fn parse_assignment_expr(registry: &mut TypeRegistry,parser: &mut Parser, left: UntypedExpr, binding_power: Bp) -> ReturnedExpression {
    spanned_led!(parser, left, {
        let operator = parser.expect_next_simple()?;

        let value = parse_expression(registry, parser, binding_power)?;

        expression::assign(left, operator, value)
    })
}


pub fn parse_array_access_expr(registry: &mut TypeRegistry,parser: &mut Parser, left: UntypedExpr, binding_power: Bp) -> ReturnedExpression {
    spanned_led!(parser, left, {
        parser.expect_next(OpenSquare)?;

        let index = parse_expression(registry, parser, COMMA.rbp)?;

        parser.expect_next(CloseSquare)?;

        expression::array_access(left, index)
    })
}


pub fn parse_null_deref(registry: &mut TypeRegistry,parser: &mut Parser, left: UntypedExpr, binding_power: Bp) -> ReturnedExpression {
    spanned_led!(parser, left, {
        parser.expect_next(TwoExcl)?;

        expression::null_deref(left)
    })
}

pub fn parse_call_expr(registry: &mut TypeRegistry,parser: &mut Parser, left: UntypedExpr, binding_power: Bp) -> ReturnedExpression {
    spanned_led!(parser, left, {
        parser.expect_next(OpenBracket)?;

        let mut args = vec![];

        while !parser.consume_if_next(CloseBracket)? {
            args.push(parse_expression(registry,parser, ASSIGNMENT.rbp)?);

            if !parser.consume_if_next(Comma)? {
                parser.expect_next(CloseBracket)?;
                break;
            }
        }

        expression::call(left, args)
    })
}