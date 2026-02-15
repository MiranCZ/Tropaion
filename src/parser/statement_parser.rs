use crate::ast::statement::{ExpressionStmt, Statement, VarDeclarationStmt};
use crate::error::parser_error::ParserError;
use crate::lexer::token::Token;
use crate::lexer::token::SimpleToken;
use crate::lexer::token::Token::SimpleTokenType;
use crate::parser::binding_power::{Bp, ASSIGNMENT};
use crate::parser::expression_parser::parse_expression;
use crate::parser::{binding_power, Parser};


pub fn parse_statement(parser: &mut Parser) -> Result<Box<dyn Statement>, ParserError> {
    let token = parser.peek()?;

    let stmnt_fn = token.statement(parser.lookup());

    if let Some(f) = stmnt_fn {
        return Ok(f(parser)?);
    }

    let expression = parse_expression(parser, binding_power::DEFAULT)?;

    parser.expect_next(SimpleTokenType(SimpleToken::Semicolon))?;

    Ok(Box::new(ExpressionStmt(expression)))
}

pub fn parse_var_declaration_stmnt(parser: &mut Parser) -> Result<Box<dyn Statement>, ParserError> {
    let token = parser.next()?;
    
    if let SimpleTokenType(t) = token {
        assert!(t == SimpleToken::Let || t == SimpleToken::Const);

        let is_const = (t == SimpleToken::Const);

        let next = parser.next()?;

        return if let Token::Identifier(v) = next {
            parser.expect_next(SimpleTokenType(SimpleToken::Assign))?;

            let value = parse_expression(parser, ASSIGNMENT)?;

            parser.expect_next(SimpleTokenType(SimpleToken::Semicolon))?;

            Ok(Box::new(VarDeclarationStmt {
                name: v,
                is_const,
                value
            }))
        } else {
            Err(ParserError::UnexpectedToken)
        }
    }

    panic!()
}