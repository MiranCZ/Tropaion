use crate::ast::statement::{Parameter, StatementBlock};
use crate::ast::statement::Statement::*;
use crate::ast::statement::Statement::ExpressionStmt;
use crate::error::parser_error::ParserError;
use crate::lexer::token::Token;
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::{Arrow, CloseBracket, Colon, Comma, Else, If, OpenBracket, Return, Semicolon, While};
use crate::lexer::token::Token::{MultilineComment, SimpleTokenType};
use crate::parser::binding_power::{Bp, ASSIGNMENT, DEFAULT};
use crate::parser::expression_parser::parse_expression;
use crate::parser::{binding_power, Parser};
use crate::parser::handlers::ReturnedStatement;
use crate::parser::type_parser::parse_type;


pub fn parse_statement(parser: &mut Parser) -> ReturnedStatement {
    let token = parser.peek()?;

    let stmnt_fn = token.statement(parser.lookup());

    if let Some(f) = stmnt_fn {
        return Ok(f(parser)?);
    }

    let expression = parse_expression(parser, binding_power::DEFAULT)?;

    parser.expect_next(SimpleToken::Semicolon)?;

    Ok(ExpressionStmt(expression))
}

pub fn parse_comment_smt(parser: &mut Parser) -> ReturnedStatement {
    let text = parser.expect_next_comment()?;
    
    Ok(CommentStmt(text))
}

pub fn parse_multiline_comment_smt(parser: &mut Parser) -> ReturnedStatement {
    let text = parser.expect_next_multiline_comment()?;

    Ok(MultilineCommentStmt(text))
}

pub fn parse_var_declaration_stmnt(parser: &mut Parser) -> ReturnedStatement {
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

    Ok(VarDeclarationStmt {
        name,
        is_const,
        value,
        explicit_type
    })
}

pub fn parse_block_stmt(parser: &mut Parser) -> ReturnedStatement {
    Ok(BlockStmt{body: _parse_block_stmt(parser)?})
}

fn _parse_block_stmt(parser: &mut Parser) -> Result<StatementBlock, ParserError> {
    parser.expect_next(SimpleToken::OpenCurly)?;

    let mut statements = vec![];

    while parser.has_next() && !parser.consume_if_next(SimpleToken::CloseCurly)? {
        statements.push(parse_statement(parser)?);
    }

    Ok(statements)
}

pub fn parse_return_stmt(parser: &mut Parser) -> ReturnedStatement {
    parser.expect_next(Return)?;

    let expr = parse_expression(parser, DEFAULT)?;

    parser.expect_next(Semicolon)?;

    Ok(ReturnStmt(expr))
}

pub fn parse_fn_declaration_stmt(parser: &mut Parser) -> ReturnedStatement {
   parser.expect_next(SimpleToken::Fn)?;

    let fn_name = parser.expect_next_identifier()?;

    parser.expect_next(SimpleToken::OpenBracket)?;

    let mut params = vec![];

    loop {
        if parser.consume_if_next(CloseBracket)? {
            break;
        }

        let param_name = parser.expect_next_identifier()?;
        parser.expect_next(Colon)?;
        let param_type = parse_type(parser, DEFAULT)?;

        params.push(Parameter{name: param_name, param_type});

        if !parser.consume_if_next(Comma)? {
            parser.expect_next(CloseBracket)?;
            break;
        }
    }

    let mut return_type = None;
    if parser.consume_if_next(Arrow)? {
        return_type = Some(parse_type(parser, DEFAULT)?);
    }

    let body = _parse_block_stmt(parser)?;

    Ok(FunctionStmt{
        name: fn_name,
        params,
        return_type,
        body
    })
}

pub fn parse_if_statement(parser: &mut Parser) -> ReturnedStatement {
    parser.expect_next(If)?;

    let parentheses = parser.consume_if_next(OpenBracket)?;

    let condition = parse_expression(parser, DEFAULT)?;

    if parentheses {
        parser.expect_next(CloseBracket)?;
    }

    let body = _parse_block_stmt(parser)?;

    let mut else_branch = None;

    if parser.consume_if_next(Else)? {
        // elif
        if parser.is_next(If)? {
            else_branch = Some(parse_if_statement(parser)?.boxed());
        } else {
            else_branch = Some(parse_block_stmt(parser)?.boxed());
        }
    }

    Ok(IfStmt {
        condition,
        body,
        else_branch
    })
}

pub fn parse_while_statement(parser: &mut Parser) -> ReturnedStatement {
    parser.expect_next(While)?;

    let parentheses = parser.consume_if_next(OpenBracket)?;

    let condition = parse_expression(parser, DEFAULT)?;

    if parentheses {
        parser.expect_next(CloseBracket)?;
    }

    let body = _parse_block_stmt(parser)?;
    
    Ok(WhileStmt {
        condition,
        body
    })
}