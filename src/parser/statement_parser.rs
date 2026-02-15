use crate::ast::statement::{ExpressionStmt, Statement, VarDeclarationStmt};
use crate::lexer::token::Token;
use crate::lexer::token::SimpleToken;
use crate::lexer::token::Token::SimpleTokenType;
use crate::parser::binding_power::{Bp, ASSIGNMENT};
use crate::parser::expression_parser::parse_expression;
use crate::parser::{binding_power, Parser};


pub fn parse_statement(parser: &mut Parser) -> Option<Box<dyn Statement>> {
    let token = parser.peek();
    if token.is_none() {
        return None;
    }
    let token = token.unwrap();

    let stmnt_fn = token.statement(parser.lookup());

    if let Some(f) = stmnt_fn {
        return Some(f(parser));
    }


    let expression = parse_expression(parser, binding_power::DEFAULT);

    if expression.is_none() {
        return None;
    }
    let expression = expression.unwrap();

    parser.expect_next(SimpleTokenType(SimpleToken::Semicolon));

    Some(Box::new(ExpressionStmt(expression)))
}

pub fn parse_var_declaration_stmnt(parser: &mut Parser) -> Box<dyn Statement> {
    let token = parser.next().unwrap();

    if let SimpleTokenType(t) = token {
        assert!(t == SimpleToken::Let || t == SimpleToken::Const);

        let is_const = (t == SimpleToken::Const);

        let next = parser.next().unwrap();

        if let Token::Identifier(v) = next {

            parser.expect_next(SimpleTokenType(SimpleToken::Assign));

            let value = parse_expression(parser, ASSIGNMENT).unwrap();

            parser.expect_next(SimpleTokenType(SimpleToken::Semicolon));

            return Box::new(VarDeclarationStmt{
                name: v,
                is_const,
                value
            })
        } else {
            panic!("Expected identifier");
        }

    }

    panic!()
}