use std::collections::HashMap;
use crate::analysis::type_registry::TypeRegistry;
use crate::ast::ast_type::AstType::{UnknownType, Void};
use crate::ast::expression::UntypedExpr;
use crate::ast::modifier::Modifier;
use crate::ast::statement::Statement::ExpressionStmt;
use crate::ast::statement::Statement::*;
use crate::ast::statement::{Parameter, StatementBlock, UntypedStmt};
use crate::error::context::ErrorContext;
use crate::error::parser_error::ParserError;
use crate::error::parser_error::ParserError::ClashingModifier;
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::{Arrow, Break, CloseBracket, Colon, Comma, Continue, Else, Enum, For, Greater, If, Less, OpenBracket, OpenCurly, Priv, Pub, Return, Semicolon, Struct, While};
use crate::parser::binding_power::{ASSIGNMENT, DEFAULT};
use crate::parser::expression_parser::parse_expression;
use crate::parser::handlers::ReturnedStatement;
use crate::parser::type_parser::parse_type;
use crate::parser::Parser;
use crate::spanned;
use crate::util::spanned::Spanned;

pub fn parse_statement(registry: &mut TypeRegistry, parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        let token = parser.peek()?;

        let stmnt_fn = token.statement(parser.lookup());

        if let Some(f) = stmnt_fn {
            return Ok(f(registry, parser)?);
        }

        let expression = match parse_expression(registry, parser, DEFAULT.rbp) {
            Ok(v) => v,
            Err(e) => {
                parser.errors.push(e);
                parser.synchronize_error(&[]);

                return Ok(UntypedStmt::err(parser.current_span()));
            }
        };

        if let Err(e) = parser.expect_next(Semicolon) {
            parser.errors.push(e);
            parser.synchronize_error(&[]);

            return Ok(UntypedStmt::err(parser.current_span()));
        }


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
    if let Err(e) = parser.expect_next(OpenCurly) {
        parser.errors.push(e);
        parser.synchronize_error(&[OpenCurly]);

        if !parser.consume_if_next(OpenCurly)? {
            return Ok(vec![]);
        }
    }

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

pub fn parse_public_modifier(registry: &mut TypeRegistry, parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {

        let from = parser.current_span().from;
        parser.expect_next(Pub)?;
        let to = parser.current_span().from;

        let mut underlying = parse_statement(registry, parser)?.node;

        match &mut underlying {
            ConstructorStmt {modifier, ..} |
            FunctionStmt {modifier,..} => {
                if !modifier.has_visibility() {
                    *modifier = modifier.with_public();
                } else {
                    // just note the error, we can keep on parsing
                    parser.errors.push(ErrorContext::new(ClashingModifier, from, to));
                }
            }
            _ => {
                // just note the error, we can keep on parsing
                parser.errors.push(ErrorContext::new(ParserError::InvalidModifier("pub".to_string()), from, to));
            }
        }

        underlying
    })
}

pub fn parse_fn_declaration_stmt(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(SimpleToken::Fn)?;

        let fn_name = parser.expect_next_identifier()?;

        let mut generics = vec![];
        if parser.consume_if_next(Less)? {

            loop {
                let name = parser.expect_next_identifier()?;

                generics.push(name);

                if !parser.consume_if_next(Comma)? {
                    parser.expect_next(Greater)?;

                    break;
                }
            }

        }
        
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
            modifier: Modifier::new(),
            generics,
            params,
            return_type,
            body
        }
    })
}

pub fn parse_constructor_stmt(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(SimpleToken::Init)?;

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

        let body = _parse_block_stmt(registry, parser)?;

        ConstructorStmt {
            modifier: Modifier::new(),
            params,
            body
        }
    })
}

pub fn parse_if_statement(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(If)?;

        let condition = parse_expression(registry, parser, DEFAULT.rbp)?;

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
    
        let condition = match parse_expression(registry, parser, DEFAULT.rbp) {
            Ok(v) => v,
            Err(e) => {
                parser.errors.push(e);
                parser.synchronize_error(&[OpenCurly]);

                UntypedExpr::err(parser.current_span())
            }
        };


        let body = _parse_block_stmt(registry, parser)?;
    
        WhileStmt {
            condition,
            body
        }
    })
}

pub fn parse_for_statement(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(For)?;

        let first = parse_statement(registry, parser)?;

        let compare = parse_expression(registry, parser, DEFAULT.rbp)?;

        parser.expect_next(Semicolon)?;

        let end = parse_expression(registry, parser, DEFAULT.rbp)?;
        let end_span = end.span;

        let body = parse_block_stmt(registry, parser)?;
        let body_span = body.span;

        BlockStmt {
            body: vec![
                first,
                Spanned::of(WhileStmt {
                    condition: compare,
                    body: vec![body, Spanned::of(ExpressionStmt(end), end_span)]
                }, body_span),
            ]
        }
    })
}

pub fn parse_continue_statement(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(Continue)?;
        parser.expect_next(Semicolon)?;

        LoopInterrupt {break_loop: false}
    })
}

pub fn parse_break_statement(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(Break)?;
        parser.expect_next(Semicolon)?;

        LoopInterrupt {break_loop: true}
    })
}


pub fn parse_struct_statement(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(Struct)?;

        let struct_name = parser.expect_next_identifier()?;

        let mut generics = vec![];
        if parser.consume_if_next(Less)? {

            loop {
                let name = parser.expect_next_identifier()?;

                generics.push(name);

                if !parser.consume_if_next(Comma)? {
                    parser.expect_next(Greater)?;

                    break;
                }
            }

        }
        
        let mut public_constructor = true;
        
        if parser.consume_if_next(Priv)? {
            public_constructor = false;
        }

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
            public_constructor,
            fields,
            body,
            generics
        }
    })
}

pub fn parse_enum_statement(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedStatement {
    spanned!(parser, {
        parser.expect_next(Enum)?;

        let enum_name = parser.expect_next_identifier()?;

        parser.expect_next(OpenBracket)?;

        let mut values = vec![];


        loop {
            if parser.consume_if_next(CloseBracket)? {
                break;
            }

            let value_name = parser.expect_next_identifier()?;
            values.push(value_name);

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

        EnumStmt {
            name: enum_name,
            values,
            body
        }
    })
}
