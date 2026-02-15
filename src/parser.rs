mod statement_parser;
pub mod handlers;
mod lookups;
pub mod binding_power;
mod expression_parser;

use std::f32::consts::E;
use crate::ast::statement::BlockStmt;
use crate::error::parser_error::ParserError;
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

    pub fn parse(&mut self) -> Result<BlockStmt, ParserError> {
        let mut body = Vec::new();

        loop {
            let token = self.peek()?;

            if token == EOF {
                break;
            }

            body.push(parse_statement(self)?);
        }

        Ok(BlockStmt{
            body
        })
    }

    pub fn next(&mut self) -> Result<Token, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::EOFError);
        }

        let token = &self.tokens[self.pos];

        self.pos += 1;

        Ok(token.clone())
    }

    pub fn expect_next(&mut self, expected: Token) -> Result<Token, ParserError> {
        let next = self.next()?;
        if next == expected {
            return Ok(next);
        }

        Err(ParserError::UnexpectedToken)
        // panic!("Expected {expected:?} got {next:?}")
    }

    pub fn peek(&self) -> Result<Token, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::EOFError);
        }

        Ok(self.tokens[self.pos].clone())
    }



}