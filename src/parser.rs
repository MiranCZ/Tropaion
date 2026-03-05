mod statement_parser;
pub mod handlers;
mod lookups;
mod type_lookups;
pub mod binding_power;
mod expression_parser;
mod type_parser;

use crate::analysis::type_registry::TypeRegistry;
use crate::ast::expression::UntypedExpr;
use crate::ast::statement::{Statement, UntypedStmt};
use crate::ast::statement::Statement::BlockStmt;
use crate::error::context::{ErrorContext, Span};
use crate::error::parser_error::ParserError;
use crate::lexer::token::Token::{Comment, Identifier, MultilineComment, NumberIntLiteral, SimpleTokenType, EOF};
use crate::lexer::token::{SimpleToken, Token};
use crate::lexer::TokenInfo;
use crate::parser::lookups::lookup::Lookup;
use crate::parser::statement_parser::parse_statement;
use crate::parser::type_lookups::type_lookup::TypeLookup;
use crate::util::spanned::Spanned;

pub struct Parser {
    lookup: Lookup,
    type_lookup: TypeLookup,
    tokens: Vec<TokenInfo>,
    pos: usize
}

impl Parser {

    pub fn new(tokens: Vec<TokenInfo>) -> Self {
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

    pub fn parse(&mut self, registry: &mut TypeRegistry) -> Result<UntypedStmt, ErrorContext<ParserError>> {
        let mut body = Vec::new();

        loop {
            let token = self.peek();

            let token = if let Ok(t) = token {
                t
            } else {
                let err = token.err().unwrap();

                return Err(ErrorContext::of(err, self.current_span()));
            };

            if token == EOF {
                break;
            }

            let from = self.current_span().from;
            let stmt = parse_statement(registry, self);
            let stmt = if let Ok(s) = stmt {
                s
            } else {
                let err = stmt.err().unwrap();

                let to = self.current_span().to;
                return Err(ErrorContext::new(err, from, to));
            };
            body.push(stmt);
        }

        Ok(Spanned::new(BlockStmt{ body }, 0, self.current_span().to))
    }

    pub fn next(&mut self) -> Result<Token, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::EOFError);
        }

        let token = &self.tokens[self.pos];

        self.pos += 1;

        Ok(token.token.clone())
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

    pub fn expect_next_comment(&mut self) -> Result<String, ParserError> {
        let next = self.next()?;
        if let Comment(v) = next {
            return Ok(v);
        }

        Err(ParserError::MismatchedTokenType{expected: "Comment".to_string(), actual: next})
    }

    pub fn expect_next_multiline_comment(&mut self) -> Result<String, ParserError> {
        let next = self.next()?;
        if let MultilineComment(v) = next {
            return Ok(v);
        }

        Err(ParserError::MismatchedTokenType{expected: "MultilineComment".to_string(), actual: next})
    }


    pub fn peek(&self) -> Result<Token, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::EOFError);
        }

        Ok(self.tokens[self.pos].token.clone())
    }

    pub fn current_span(&self) -> Span {
        if self.pos >= self.tokens.len() {
            if self.tokens.len() == 0 {
                return Span::new(0, 0);
            }

            return self.tokens[0].span;
        }

        self.tokens[self.pos].span
    }



}