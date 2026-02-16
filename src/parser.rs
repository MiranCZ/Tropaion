mod statement_parser;
pub mod handlers;
mod lookups;
mod type_lookups;
pub mod binding_power;
mod expression_parser;
mod type_parser;

use crate::ast::statement::BlockStmt;
use crate::error::parser_error::ParserError;
use crate::lexer::token::Token::{Identifier, NumberIntLiteral, SimpleTokenType, EOF};
use crate::lexer::token::{SimpleToken, Token};
use crate::parser::lookups::lookup::Lookup;
use crate::parser::statement_parser::parse_statement;
use crate::parser::type_lookups::type_lookup::TypeLookup;

pub struct Parser {
    lookup: Lookup,
    type_lookup: TypeLookup,
    tokens: Vec<Token>,
    pos: usize
}

impl Parser {

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            lookup: Lookup::new(),
            type_lookup: TypeLookup::new(),
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

    pub fn has_next(&self) -> bool {
        self.pos < self.tokens.len()
    }

    pub fn is_next(&self, expected: SimpleToken) -> Result<bool, ParserError> {
        if let SimpleTokenType(v) = self.peek()? && v == expected {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn consume_if_next(&mut self, expected: SimpleToken) -> Result<bool, ParserError> {
        if self.is_next(expected)? {
            self.next()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn expect_next(&mut self, expected: SimpleToken) -> Result<Token, ParserError> {
        let next = self.next()?;
        if let SimpleTokenType(v) = next && v == expected {
            return Ok(next);
        }

        Err(ParserError::UnexpectedToken{expected: SimpleTokenType(expected), actual: next})
    }

    pub fn expect_next_simple(&mut self) -> Result<SimpleToken, ParserError> {
        let next = self.next()?;
        if let SimpleTokenType(v) = next {
            return Ok(v);
        }

        Err(ParserError::MismatchedTokenType{expected: "SimpleToken".to_string(), actual: next})
    }

    pub fn expect_next_identifier(&mut self) -> Result<String, ParserError> {
        let next = self.next()?;
        if let Identifier(v) = next {
            return Ok(v);
        }

        Err(ParserError::MismatchedTokenType{expected: "Identifier".to_string(), actual: next})
    }

    pub fn expect_next_int(&mut self) -> Result<i32, ParserError> {
        let next = self.next()?;
        if let NumberIntLiteral(v) = next {
            return Ok(v);
        }

        Err(ParserError::MismatchedTokenType{expected: "NumberIntLiteral".to_string(), actual: next})
    }

    pub fn peek(&self) -> Result<Token, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::EOFError);
        }

        Ok(self.tokens[self.pos].clone())
    }



}