pub mod lookup;

use crate::ast::expression::{
    Expression, FloatLiteralExpr, IdentifierExpr, IntLiteralExpr, StringLiteralExpr,
};
use crate::error::parser_error::ParserError;
use crate::lexer::token::Token;
use crate::lexer::token::Token::*;
use crate::parser::handlers::{LedInfo, NudHandler, StatementHandler};
use crate::parser::lookups::lookup::Lookup;
use crate::parser::statement_parser::{parse_comment_smt, parse_multiline_comment_smt};
use crate::parser::Parser;

impl Token {
    pub fn nud(&self, lookup: &Lookup) -> Option<NudHandler> {
        fn handle_literal(parser: &mut Parser) -> Result<Box<dyn Expression>, ParserError> {
            let token = parser.next()?;

            Ok(match token {
                Identifier(v) => Box::new(IdentifierExpr(v.clone())),
                NumberIntLiteral(v) => Box::new(IntLiteralExpr(v)),
                NumberFloatLiteral(v) => Box::new(FloatLiteralExpr(v)),
                StringLiteral(v) => Box::new(StringLiteralExpr(v.clone())),

                _ => panic!()
            })
        }

        match self {
            Identifier(_) => Some(handle_literal),
            NumberIntLiteral(_) => Some(handle_literal),
            NumberFloatLiteral(_) => Some(handle_literal),
            StringLiteral(_) => Some(handle_literal),

            SimpleTokenType(t) => lookup.nud_lookup.get(t).copied(),

            _ => None,
        }
    }

    pub fn led(&self, lookup: &Lookup) -> Option<LedInfo> {
        if let SimpleTokenType(t) = self {
            return lookup.led_lookup.get(t).copied();
        }

        None
    }

    pub fn statement(&self, lookup: &Lookup) -> Option<StatementHandler> {
        if let Comment(_) = self {
            return Some(parse_comment_smt)
        }
        if let MultilineComment(_) = self {
            return Some(parse_multiline_comment_smt)
        }
        
        if let SimpleTokenType(t) = self {
            return lookup.statement_lookup.get(t).copied();
        }

        None
    }
}
