mod statement_parser;
pub mod handlers;
mod lookups;
pub mod binding_power;
mod expression_parser;

use std::f32::consts::E;
use crate::ast::statement::BlockStmt;
use crate::lexer::token::Token;
use crate::lexer::token::Token::EOF;
use crate::parser::lookups::lookup::Lookup;
use crate::parser::statement_parser::parse_statement;

pub struct Parser {
    lookup: Lookup,
    tokens: Vec<Token>,
    pos: usize
}

impl Parser {

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            lookup: Lookup::new(),
            tokens,
            pos: 0
        }
    }

    pub fn lookup(&self) -> &Lookup {
        &self.lookup
    }

    pub fn parse(&mut self) -> BlockStmt {
        let mut body = Vec::new();

        while let Some(token) = self.peek() {
            if token == EOF {
                break;
            }

            body.push(parse_statement(self).unwrap());
        }

        return BlockStmt{
            body
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.pos >= self.tokens.len() {
            return None;
        }

        let token = &self.tokens[self.pos];

        self.pos += 1;

        Some(token.clone())
    }

    pub fn expect_next(&mut self, expected: Token) -> Option<Token> {
        let next = self.next();
        if let Some(token) = next.clone() && token == expected {
            return Some(token);
        }

        panic!("Expected {expected:?} got {next:?}")
    }

    pub fn peek(&self) -> Option<Token> {
        if self.pos >= self.tokens.len() {
            return None;
        }

        Some(self.tokens[self.pos].clone())
    }



}