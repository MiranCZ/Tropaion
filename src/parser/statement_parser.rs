use crate::analysis::type_registry::TypeRegistry;
use crate::ast::ast_type::AstType::Void;
use crate::ast::statement::{Parameter, StatementBlock};
use crate::ast::statement::Statement::*;
use crate::ast::statement::Statement::ExpressionStmt;
use crate::error::context::ErrorContext;
use crate::error::parser_error::ParserError;
use crate::lexer::token::Token;
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::{Arrow, CloseBracket, Colon, Comma, Else, If, OpenBracket, OpenCurly, Return, Semicolon, Struct, While};
use crate::lexer::token::Token::{MultilineComment, SimpleTokenType};
use crate::parser::binding_power::{Bp, ASSIGNMENT, DEFAULT};
use crate::parser::expression_parser::parse_expression;
use crate::parser::{binding_power, Parser};
use crate::parser::handlers::ReturnedStatement;
use crate::parser::type_parser::parse_type;
use crate::util::spanned::Spanned;
use crate::spanned;

pub fn parse_statement(registry: &mut TypeRegistry, parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        let token = parser.peek()?;

        let stmnt_fn = token.statement(parser.lookup());

        if let Some(f) = stmnt_fn {
            return Ok(f(registry, parser)?);
        }

        let expression = parse_expression(registry, parser, DEFAULT.rbp)?;

        parser.expect_next(SimpleToken::Semicolon)?;

        ExpressionStmt(expression)
    })
}

pub fn parse_comment_smt(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        let text = parser.expect_next_comment()?;

        CommentStmt(text)
    })
}

pub fn parse_multiline_comment_smt(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        let text = parser.expect_next_multiline_comment()?;

        MultilineCommentStmt(text)
    })
}

pub fn parse_var_declaration_stmnt(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        let token = parser.expect_next_simple()?;

        assert!(token == SimpleToken::Let || token == SimpleToken::Const);

        let is_const = (token == SimpleToken::Const);
        let name = parser.expect_next_identifier()?;

        let mut explicit_type = None;
        if parser.consume_if_next(Colon)? {
            explicit_type = Some(parse_type(registry, parser, DEFAULT.rbp)?);
        }

        parser.expect_next(SimpleToken::Assign)?;

        let value = parse_expression(registry, parser, ASSIGNMENT.rbp)?;

        parser.expect_next(SimpleToken::Semicolon)?;

        VarDeclarationStmt {
            name,
            is_const,
            value,
            explicit_type
        }
    })
}

pub fn parse_block_stmt(registry: &mut TypeRegistry, parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        BlockStmt{body: _parse_block_stmt(registry, parser)?}
    })
}

fn _parse_block_stmt(registry: &mut TypeRegistry,parser: &mut Parser) -> Result<StatementBlock<()>, ErrorContext<ParserError>> {
    parser.expect_next(SimpleToken::OpenCurly)?;

    let mut statements = vec![];

    while parser.has_next() && !parser.consume_if_next(SimpleToken::CloseCurly)? {
        statements.push(parse_statement(registry, parser)?);
    }

    Ok(statements)
}

pub fn parse_return_stmt(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(Return)?;

        let expr = parse_expression(registry, parser, DEFAULT.rbp)?;

        parser.expect_next(Semicolon)?;

        ReturnStmt(expr)
    })
}

pub fn parse_fn_declaration_stmt(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
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
            let param_type = parse_type(registry, parser, DEFAULT.rbp)?;

            params.push(Parameter{name: param_name, param_type});

            if !parser.consume_if_next(Comma)? {
                parser.expect_next(CloseBracket)?;
                break;
            }
        }

        let mut return_type = registry.register(Void);
        if parser.consume_if_next(Arrow)? {
            return_type = parse_type(registry, parser, DEFAULT.rbp)?;
        }

        let body = _parse_block_stmt(registry, parser)?;

        FunctionStmt{
            name: fn_name,
            params,
            return_type,
            body
        }
    })
}

pub fn parse_if_statement(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(If)?;

        let parentheses = parser.consume_if_next(OpenBracket)?;

        let condition = parse_expression(registry, parser, DEFAULT.rbp)?;

        if parentheses {
            parser.expect_next(CloseBracket)?;
        }

        let body = _parse_block_stmt(registry, parser)?;

        let mut else_branch = None;

        if parser.consume_if_next(Else)? {
            // elif
            if parser.is_next(If)? {
                else_branch = Some(parse_if_statement(registry, parser)?.boxed());
            } else {
                else_branch = Some(parse_block_stmt(registry, parser)?.boxed());
            }
        }

        IfStmt {
            condition,
            body,
            else_branch
        }
    })
}

pub fn parse_while_statement(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(While)?;
    
        let parentheses = parser.consume_if_next(OpenBracket)?;
    
        let condition = parse_expression(registry, parser, DEFAULT.rbp)?;
    
        if parentheses {
            parser.expect_next(CloseBracket)?;
        }
    
        let body = _parse_block_stmt(registry, parser)?;
    
        WhileStmt {
            condition,
            body
        }
    })
}

pub fn parse_struct_statement(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(Struct)?;

        let struct_name = parser.expect_next_identifier()?;

        parser.expect_next(SimpleToken::OpenBracket)?;

        let mut fields = vec![];

        loop {
            if parser.consume_if_next(CloseBracket)? {
                break;
            }

            let param_name = parser.expect_next_identifier()?;
            parser.expect_next(Colon)?;
            let param_type = parse_type(registry, parser, DEFAULT.rbp)?;

            fields.push(Parameter{name: param_name, param_type});

            if !parser.consume_if_next(Comma)? {
                parser.expect_next(CloseBracket)?;
                break;
            }
        }

        let body;

        if parser.is_next(OpenCurly)? {
            body = _parse_block_stmt(registry, parser)?;
        }  else {

            // a struct without methods
            parser.expect_next(Semicolon)?;
            body = vec![];
        }

        StructStmt {
            name: struct_name,
            fields,
            body
        }
    })
}