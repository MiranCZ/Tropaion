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

                return Err(err);
            };

            if token == EOF {
                break;
            }

            let stmt = parse_statement(registry, self);
            let stmt = if let Ok(s) = stmt {
                s
            } else {
                let err = stmt.err().unwrap();

                return Err(err);
            };
            body.push(stmt);
        }

        Ok(Spanned::new(BlockStmt{ body }, 0, self.current_span().to))
    }

    pub fn next(&mut self) -> Result<Token, ErrorContext<ParserError>> {
        if self.pos >= self.tokens.len() {
            return Err(ErrorContext::of(ParserError::EOFError, self.current_span()));
        }

        let token = &self.tokens[self.pos];

        self.pos += 1;

        Ok(token.token.clone())
    }

    fn next_spanned(&mut self) -> Result<TokenInfo, ErrorContext<ParserError>> {
        if self.pos >= self.tokens.len() {
            return Err(ErrorContext::of(ParserError::EOFError, self.current_span()));
        }

        let token = &self.tokens[self.pos];

        self.pos += 1;

        Ok(token.clone())
    }

    pub fn has_next(&self) -> bool {
        self.pos < self.tokens.len()
    }

    pub fn is_next(&self, expected: SimpleToken) -> Result<bool, ErrorContext<ParserError>> {
        if let SimpleTokenType(v) = self.peek()? && v == expected {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn consume_if_next(&mut self, expected: SimpleToken) -> Result<bool, ErrorContext<ParserError>> {
        if self.is_next(expected)? {
            self.next()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn expect_next(&mut self, expected: SimpleToken) -> Result<Token, ErrorContext<ParserError>> {
        let next = self.next_spanned()?;
        if let SimpleTokenType(v) = next.token && v == expected {
            return Ok(next.token);
        }

        Err(ErrorContext::of(ParserError::UnexpectedToken{expected: SimpleTokenType(expected), actual: next.token}, next.span))
    }

    pub fn expect_next_simple(&mut self) -> Result<SimpleToken, ErrorContext<ParserError>> {
        let next = self.next_spanned()?;
        if let SimpleTokenType(v) = next.token {
            return Ok(v);
        }

        Err(ErrorContext::of(ParserError::MismatchedTokenType{expected: "SimpleToken".to_string(), actual: next.token}, next.span))
    }

    pub fn expect_next_identifier(&mut self) -> Result<String, ErrorContext<ParserError>> {
        let next = self.next_spanned()?;
        if let Identifier(v) = next.token {
            return Ok(v);
        }

        Err(ErrorContext::of(ParserError::MismatchedTokenType{expected: "Identifier".to_string(), actual: next.token}, next.span))
    }

    pub fn expect_next_int(&mut self) -> Result<i32, ErrorContext<ParserError>> {
        let next = self.next_spanned()?;
        if let NumberIntLiteral(v) = next.token {
            return Ok(v);
        }

        Err(ErrorContext::of(ParserError::MismatchedTokenType{expected: "NumberIntLiteral".to_string(), actual: next.token}, next.span))
    }

    pub fn expect_next_comment(&mut self) -> Result<String, ErrorContext<ParserError>> {
        let next = self.next_spanned()?;
        if let Comment(v) = next.token {
            return Ok(v);
        }

        Err(ErrorContext::of(ParserError::MismatchedTokenType{expected: "Comment".to_string(), actual: next.token}, next.span))
    }

    pub fn expect_next_multiline_comment(&mut self) -> Result<String, ErrorContext<ParserError>> {
        let next = self.next_spanned()?;
        if let MultilineComment(v) = next.token {
            return Ok(v);
        }

        Err(ErrorContext::of(ParserError::MismatchedTokenType{expected: "MultilineComment".to_string(), actual: next.token}, next.span))
    }


    pub fn peek(&self) -> Result<Token, ErrorContext<ParserError>> {
        if self.pos >= self.tokens.len() {
            return Err(ErrorContext::of(ParserError::EOFError, self.current_span()));
        }

        Ok(self.tokens[self.pos].token.clone())
    }

    pub fn current_span(&self) -> Span {
        if self.pos >= self.tokens.len() {
            if self.tokens.len() == 0 {
                return Span::new(0, 0);
            }

            return self.tokens[self.tokens.len()-1].span;
        }

        self.tokens[self.pos].span
    }



}