use crate::ast::statement::{ExpressionStmt, Statement, VarDeclarationStmt};
use crate::error::parser_error::ParserError;
use crate::lexer::token::Token;
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::Colon;
use crate::lexer::token::Token::SimpleTokenType;
use crate::parser::binding_power::{Bp, ASSIGNMENT, DEFAULT};
use crate::parser::expression_parser::parse_expression;
use crate::parser::{binding_power, Parser};
use crate::parser::type_parser::parse_type;

pub fn parse_statement(parser: &mut Parser) -> Result<Box<dyn Statement>, ParserError> {
    let token = parser.peek()?;

    let stmnt_fn = token.statement(parser.lookup());

    if let Some(f) = stmnt_fn {
        return Ok(f(parser)?);
    }

    let expression = parse_expression(parser, binding_power::DEFAULT)?;

    parser.expect_next(SimpleToken::Semicolon)?;

    Ok(Box::new(ExpressionStmt(expression)))
}

pub fn parse_var_declaration_stmnt(parser: &mut Parser) -> Result<Box<dyn Statement>, ParserError> {
    let token = parser.expect_next_simple()?;
   
    assert!(token == SimpleToken::Let || token == SimpleToken::Const);

    let is_const = (token == SimpleToken::Const);
    let name = parser.expect_next_identifier()?;

    let mut explicit_type = None;
    if parser.consume_if_next(Colon)? {
        explicit_type = Some(parse_type(parser, DEFAULT)?);
    }

    parser.expect_next(SimpleToken::Assign)?;

    let value = parse_expression(parser, ASSIGNMENT)?;

    parser.expect_next(SimpleToken::Semicolon)?;

    Ok(Box::new(VarDeclarationStmt {
        name,
        is_const,
        value, 
        explicit_type
    }))
}