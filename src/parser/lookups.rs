pub mod lookup;

use crate::ast::expression::{
    Expression, FloatLiteralExpr, IdentifierExpr, IntLiteralExpr, StringLiteralExpr,
};
use crate::ast::statement::Statement;
use crate::lexer::token::Token;
use crate::lexer::token::Token::*;
use crate::parser::handlers::{LedInfo, NudHandler, StatementHandler};
use crate::parser::lookups::lookup::Lookup;
use crate::parser::Parser;

impl Token {
    pub fn nud(&self, lookup: &Lookup) -> Option<NudHandler> {
        fn handle_literal(parser: &mut Parser) -> Box<dyn Expression> {
            let token = parser.next().unwrap();

            match token {
                Identifier(v) => Box::new(IdentifierExpr(v.clone())),
                NumberIntLiteral(v) => Box::new(IntLiteralExpr(v)),
                NumberFloatLiteral(v) => Box::new(FloatLiteralExpr(v)),
                StringLiteral(v) => Box::new(StringLiteralExpr(v.clone())),

                _ => panic!()
            }
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
        if let SimpleTokenType(t) = self {
            return lookup.statement_lookup.get(t).copied();
        }

        None
    }
}
